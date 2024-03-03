use crate::database::RedisDatabase;
use crate::error::ConvertError;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{redis, Connection};
use error_stack::{Report, ResultExt};
use kernel::interface::database::DatabaseConnection;
use kernel::interface::job::{
    AsyncWork, DestructQueueInfo, ErrorOperation, ErroredInfo, MQConfig, MessageQueue, QueueInfo,
};
use kernel::KernelError;
use redis::streams::StreamReadOptions;
use redis::{RedisResult, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};
use uuid::Uuid;

#[derive(Debug)]
struct QueueData<T> {
    id: String,
    delivered_count: i64,
    info: QueueInfo<T>,
}

pub struct RedisMessageQueue<T>
where
    T: 'static + Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    name: String,
    db: RedisDatabase,
    config: MQConfig,
    worker_process: Arc<Box<dyn Fn(T) -> AsyncWork + Send + Sync>>,
    _data_type: PhantomData<T>,
}

impl<T> RedisMessageQueue<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    #[tracing::instrument(skip(db, block))]
    async fn listen(
        db: RedisDatabase,
        name: String,
        config: MQConfig,
        block: Arc<Box<impl Fn(T) -> AsyncWork + Sync + Send + ?Sized>>,
    ) {
        let member_name = format!("consumer:{}", Uuid::new_v4());
        loop {
            let QueueData {
                id,
                delivered_count,
                info,
            } = {
                let mut con = match db.transact().await {
                    Ok(con) => con,
                    Err(report) => {
                        error!("{report:?}");
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };
                let mut result = RedisJobInternal::pop_pending::<T>(
                    &mut con,
                    &name,
                    &member_name,
                    &config.retry_delay,
                )
                .await;
                if result.is_err() || result.as_ref().is_ok_and(Option::is_none) {
                    result = RedisJobInternal::pop_to_process(&mut con, &name, &member_name).await;
                }
                match result {
                    Ok(Some(data)) => data,
                    Ok(None) => continue,
                    Err(report) => {
                        error!("{report:?}");
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
            };
            debug!("Processing Id: {id}, TryCount: {delivered_count}");
            let DestructQueueInfo { id: uuid, data }: DestructQueueInfo<T> = info.into_destruct();
            let result = block(data.clone()).await;
            {
                let transact = db.transact().await;
                let mut con = match transact {
                    Ok(con) => con,
                    Err(report) => {
                        error!("{report:?}");
                        continue;
                    }
                };

                if let Err(report) = result {
                    if delivered_count > config.max_retry.into() {
                        if let Err(report) = RedisJobInternal::push_failed_info(
                            &mut con,
                            &name,
                            format!(
                                "{:?}",
                                report.attach_printable("Task failed or 3 time delayed")
                            ),
                            uuid,
                            data,
                        )
                        .await
                        {
                            error!("{report:?}");
                        }
                        error!("Failed Id: {id}, TryCount: {delivered_count}");
                    } else if let ErrorOperation::Delay = report.current_context() {
                        if let Err(report) = RedisJobInternal::push_delayed_info(
                            &mut con,
                            &name,
                            uuid,
                            data,
                            format!("{report:?}"),
                        )
                        .await
                        {
                            error!("{report:?}");
                        }
                        warn!("Delayed Id: {id}, TryCount: {delivered_count}, Report: {report:?}");
                        continue;
                    }
                } else {
                    debug!("Done Id: {id}, TryCount: {delivered_count}");
                }
                if let Err(report) = RedisJobInternal::mark_done(&mut con, &name, &id).await {
                    error!("{report:?}");
                } else if delivered_count > 0 {
                    if let Err(report) =
                        RedisJobInternal::remove_delayed_info(&mut con, &name, &uuid).await
                    {
                        error!("{report:?}");
                    };
                };
            }
        }
    }
}

#[async_trait::async_trait]
impl<T> MessageQueue<T> for RedisMessageQueue<T>
where
    T: 'static + Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    type DatabaseConnection = RedisDatabase;

    fn new<F>(db: Self::DatabaseConnection, name: &str, config: MQConfig, process: F) -> Self
    where
        F: 'static + Fn(T) -> AsyncWork + Sync + Send,
    {
        Self {
            name: name.to_string(),
            db,
            config,
            worker_process: Arc::new(Box::new(process)),
            _data_type: PhantomData,
        }
    }

    fn start_workers(&self) {
        for _ in 1..self.config.worker_count {
            let db = self.db.clone();
            let process = Arc::clone(&self.worker_process);
            let name = self.name.clone();
            let config = self.config.clone();
            tokio::spawn(async move {
                RedisMessageQueue::listen(db, name, config, process).await;
            });
        }
    }

    async fn queue(&self, info: &QueueInfo<T>) -> error_stack::Result<(), KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::insert_waiting(&mut con, name, info).await
    }

    async fn get_queued_len(&self) -> error_stack::Result<usize, KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::get_wait_len(&mut con, name)
            .await
            .and_then(|size| usize::try_from(size).change_context_lazy(|| KernelError::Internal))
    }

    async fn get_delayed(
        &self,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::get_delayed_info(&mut con, name, size, offset).await
    }

    async fn get_delayed_len(&self) -> error_stack::Result<usize, KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::get_delayed_len(&mut con, name)
            .await
            .and_then(|size| usize::try_from(size).change_context_lazy(|| KernelError::Internal))
    }

    async fn get_failed(
        &self,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::get_failed_info(&mut con, name, size, offset).await
    }

    async fn get_failed_len(&self) -> error_stack::Result<usize, KernelError> {
        let name = &self.name;
        let mut con = self.db.transact().await?;
        RedisJobInternal::get_failed_len(&mut con, name)
            .await
            .and_then(|size| usize::try_from(size).change_context_lazy(|| KernelError::Internal))
    }
}

const QUEUE_FIELD: &str = "info";

fn group(name: &str) -> String {
    format!("g:{name}")
}

fn failed(name: &str) -> String {
    format!("failed:{name}")
}

fn delayed(name: &str) -> String {
    format!("delayed:{name}")
}

fn parse_error(value: impl Debug) -> Report<KernelError> {
    Report::new(KernelError::Internal)
        .attach_printable(format!("Failed to parse received data. {value:?}"))
}

pub(in crate::database) struct RedisJobInternal;

impl RedisJobInternal {
    async fn create_group(con: &mut Connection, name: &str) -> RedisResult<Value> {
        con.xgroup_create_mkstream(name, &group(name), 0).await
    }

    async fn insert_waiting<T: Serialize>(
        con: &mut Connection,
        name: &str,
        info: &QueueInfo<T>,
    ) -> error_stack::Result<(), KernelError> {
        // Ignore error
        let _ = Self::create_group(con, name).await;
        let serialize = serde_json::to_string(info)
            .map_err(|e| Report::new(e).change_context(KernelError::Internal))?;
        con.xadd(name, "*", &[(QUEUE_FIELD, &serialize)])
            .await
            .convert_error()
    }

    async fn pop_to_process<T>(
        con: &mut Connection,
        name: &str,
        member: &str,
    ) -> error_stack::Result<Option<QueueData<T>>, KernelError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let options = StreamReadOptions::default()
            .block(1000)
            .count(1)
            .group(group(name), member);
        let result: Value = con
            .xread_options(&[name], &[">"], &options)
            .await
            .convert_error()?;
        let bulk = match result {
            Value::Bulk(bulk) => bulk,
            Value::Nil => return Ok(None),
            _ => return Err(parse_error(result)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Data(_name), Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let (id, bulk) = match bulk.as_slice() {
            [Value::Data(id), Value::Bulk(bulk)] => (id, bulk),
            _ => return Err(parse_error(bulk)),
        };
        let data = match bulk.as_slice() {
            [Value::Data(_field), Value::Data(data)] => data,
            _ => return Err(parse_error(bulk)),
        };
        Ok(Some(QueueData {
            id: from_utf8(id)
                .change_context_lazy(|| KernelError::Internal)?
                .to_string(),
            delivered_count: 0,
            info: serde_json::from_slice(data).change_context_lazy(|| KernelError::Internal)?,
        }))
    }

    async fn mark_done(
        con: &mut Connection,
        name: &str,
        id: &str,
    ) -> error_stack::Result<(), KernelError> {
        con.xack(name, &group(name), &[id]).await.convert_error()?;
        con.xdel(name, &[id]).await.convert_error()
    }

    async fn pop_pending<T>(
        con: &mut Connection,
        name: &str,
        own_member: &str,
        time_millis: &i32,
    ) -> error_stack::Result<Option<QueueData<T>>, KernelError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Ignore error
        let _ = Self::create_group(con, name).await;
        let group = group(name);
        let value: Value = redis::cmd("XPENDING")
            .arg(name)
            .arg(&group)
            .arg("IDLE")
            .arg(time_millis)
            .arg("-")
            .arg("+")
            .arg(1) // count
            .query_async(con)
            .await
            .convert_error()?;

        let bulk = match value {
            Value::Bulk(bulk) => bulk,
            _ => return Err(parse_error(value)),
        };
        if bulk.is_empty() {
            return Ok(None);
        }
        let bulk = match bulk.as_slice() {
            [Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let (id, count) = match bulk.as_slice() {
            [Value::Data(id), Value::Data(_original_owner), _time, Value::Int(count)] => (
                from_utf8(id)
                    .change_context_lazy(|| KernelError::Internal)?
                    .to_string(),
                *count,
            ),
            _ => return Err(parse_error(bulk)),
        };

        let result: Value = con
            .xclaim(name, &group, own_member, time_millis, &[&id])
            .await
            .convert_error()?;

        let bulk = match result {
            Value::Bulk(bulk) => bulk,
            _ => return Err(parse_error(result)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Data(_id), Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        match bulk.as_slice() {
            [Value::Data(_field), Value::Data(data)] => {
                let info: QueueInfo<T> =
                    serde_json::from_slice(data).change_context_lazy(|| KernelError::Internal)?;

                Ok(Some(QueueData {
                    id,
                    delivered_count: count,
                    info,
                }))
            }
            _ => Err(parse_error(bulk)),
        }
    }

    async fn push_delayed_info<T: Serialize>(
        con: &mut Connection,
        name: &str,
        id: Uuid,
        data: T,
        stack_trace: String,
    ) -> error_stack::Result<(), KernelError> {
        let string_id = id.to_string();
        let info = ErroredInfo::new(id, data, stack_trace);
        let raw = serde_json::to_string(&info).change_context_lazy(|| KernelError::Internal)?;
        con.hset(&delayed(name), &string_id, &raw)
            .await
            .convert_error()
    }

    async fn remove_delayed_info(
        con: &mut Connection,
        name: &str,
        id: &Uuid,
    ) -> error_stack::Result<(), KernelError> {
        con.hdel(&delayed(name), &id.to_string())
            .await
            .convert_error()
    }

    async fn get_delayed_info<T: for<'de> Deserialize<'de>>(
        con: &mut Connection,
        name: &str,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError> {
        Self::get_info_from_hash(con, &delayed(name), size, offset).await
    }

    async fn get_delayed_len(
        con: &mut Connection,
        name: &str,
    ) -> error_stack::Result<i64, KernelError> {
        let delayed = delayed(name);
        let result: Value = con.hlen(&delayed).await.convert_error()?;
        if let Value::Int(size) = result {
            Ok(size)
        } else {
            Err(Report::new(KernelError::Internal)
                .attach_printable(format!("Failed to get size. target: {delayed}")))
        }
    }

    async fn push_failed_info<T: Serialize>(
        con: &mut Connection,
        name: &str,
        info: String,
        uuid: Uuid,
        data: T,
    ) -> error_stack::Result<(), KernelError> {
        let raw_uuid = uuid.to_string();
        let data = ErroredInfo::new(uuid, data, info);
        let raw = serde_json::to_string(&data).change_context_lazy(|| KernelError::Internal)?;
        con.hset(&failed(name), &raw_uuid, &raw)
            .await
            .convert_error()
    }

    async fn get_failed_info<T: for<'de> Deserialize<'de>>(
        con: &mut Connection,
        name: &str,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError> {
        Self::get_info_from_hash(con, &failed(name), size, offset).await
    }

    async fn get_failed_len(
        con: &mut Connection,
        name: &str,
    ) -> error_stack::Result<i64, KernelError> {
        let failed = failed(name);
        let result: Value = con.hlen(&failed).await.convert_error()?;
        if let Value::Int(size) = result {
            Ok(size)
        } else {
            Err(Report::new(KernelError::Internal)
                .attach_printable(format!("Failed to get size. target: {failed}")))
        }
    }

    async fn get_wait_len(
        con: &mut Connection,
        name: &str,
    ) -> error_stack::Result<i64, KernelError> {
        let result: Value = con.xlen(name).await.convert_error()?;
        if let Value::Int(size) = result {
            Ok(size)
        } else {
            Err(Report::new(KernelError::Internal)
                .attach_printable(format!("Failed to get size. target: {name}")))
        }
    }

    async fn get_info_from_hash<T: for<'de> Deserialize<'de>>(
        con: &mut Connection,
        name: &str,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<T>, KernelError> {
        if *size <= 0 {
            return Ok(vec![]);
        }
        let result: Value = redis::cmd("HSCAN")
            .arg(name)
            .arg(offset)
            .arg("COUNT")
            .arg(size)
            .query_async(con)
            .await
            .convert_error()?;
        let bulk = match result {
            Value::Bulk(bulk) => bulk,
            _ => return Err(parse_error(result)),
        };
        let bulk = match bulk.as_slice() {
            [Value::Data(_offset), Value::Bulk(bulk)] => bulk,
            _ => return Err(parse_error(bulk)),
        };
        let usize = usize::try_from(*size).change_context_lazy(|| KernelError::Internal)?;
        // HSCAN may return more than size
        bulk.chunks(2)
            .take(usize)
            .map(|pair| match pair {
                [Value::Data(_id), Value::Data(data)] => {
                    let info = serde_json::from_slice(data)
                        .change_context_lazy(|| KernelError::Internal)?;
                    Ok(info)
                }
                _ => Err(parse_error(pair)),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::database::redis::mq::{QueueData, RedisJobInternal, RedisMessageQueue};
    use crate::database::RedisDatabase;
    use error_stack::Report;
    use kernel::interface::database::DatabaseConnection;
    use kernel::interface::job::ErrorOperation::Delay;
    use kernel::interface::job::{MQConfig, MessageQueue, QueueInfo};
    use kernel::KernelError;
    use rand::random;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::info;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestData {
        a: String,
    }

    #[test_with::env(REDIS_TEST)]
    #[tokio::test]
    async fn test_internal() -> error_stack::Result<(), KernelError> {
        let db = RedisDatabase::new()?;
        let mut con = db.transact().await?;
        let name = "test";
        let member = "member";
        let data = TestData {
            a: "testtss".to_string(),
        };
        let info = QueueInfo::new(Uuid::new_v4(), data);
        RedisJobInternal::insert_waiting(&mut con, name, &info).await?;
        let result: QueueData<TestData> = RedisJobInternal::pop_to_process(&mut con, name, member)
            .await
            .and_then(|option| option.ok_or_else(|| Report::new(KernelError::Internal)))?;
        println!("result: {result:?}");

        sleep(Duration::from_secs(1)).await;
        let pending: Option<QueueData<TestData>> =
            RedisJobInternal::pop_pending(&mut con, name, member, &500).await?;
        println!("result: {pending:?}");

        RedisJobInternal::mark_done(&mut con, name, &result.id).await?;
        Ok(())
    }

    #[ignore]
    #[test_with::env(REDIS_TEST)]
    #[tokio::test]
    async fn test_mq() -> error_stack::Result<(), KernelError> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
        let db = RedisDatabase::new()?;
        let name = "test";
        let config = MQConfig {
            worker_count: 5,
            max_retry: 3,
            retry_delay: 1000,
        };
        let mq = RedisMessageQueue::new(db.clone(), name, config, |data: TestData| {
            Box::pin(async move {
                info!("data: {data:?}");
                sleep(Duration::from_millis(20)).await;
                // Delayed in 50%
                if random() {
                    Ok(())
                } else {
                    Err(Report::new(Delay))
                }
            })
        });

        mq.start_workers();

        for i in 0..1000 {
            let data = TestData {
                a: format!("test:{i}"),
            };
            let data = QueueInfo::new(Uuid::new_v4(), data);
            // Queue
            mq.queue(&data).await?;
        }

        let mut con = db.transact().await.unwrap();
        loop {
            let wait = RedisJobInternal::get_wait_len(&mut con, name).await?;
            let delayed = RedisJobInternal::get_delayed_len(&mut con, name).await?;
            let failed = RedisJobInternal::get_failed_len(&mut con, name).await?;
            info!("Count: {wait}, Delayed: {delayed}, Failed: {failed}");
            sleep(Duration::from_secs(1)).await;
        }
    }
}
