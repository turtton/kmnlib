use crate::database::DatabaseConnection;
use crate::KernelError;
use destructure::{Destructure, Mutation};
use error_stack::Context;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use uuid::Uuid;
use vodca::References;

#[derive(Debug)]
pub enum ErrorOperation {
    Delay,
    Failed,
}

impl Display for ErrorOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorOperation::Delay => write!(f, "Queue delayed"),
            ErrorOperation::Failed => write!(f, "Queue failed"),
        }
    }
}

impl Context for ErrorOperation {}

#[derive(Debug, Serialize, Deserialize, Destructure)]
pub struct QueueInfo<T> {
    id: Uuid,
    data: T,
}

impl<T> QueueInfo<T> {
    pub fn new(id: Uuid, data: T) -> Self {
        Self { id, data }
    }
}

impl<T> From<T> for QueueInfo<T> {
    fn from(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            data: value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, References, Destructure)]
pub struct ErroredInfo<T> {
    id: Uuid,
    data: T,
    stack_trace: String,
}

impl<T> ErroredInfo<T> {
    pub fn new(id: Uuid, data: T, stack_trace: String) -> Self {
        Self {
            id,
            data,
            stack_trace,
        }
    }
}

#[derive(Debug, Clone, References, Mutation)]
pub struct MQConfig {
    worker_count: i32,
    max_retry: i32,
    retry_delay: Duration,
}

impl Default for MQConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            max_retry: 3,
            retry_delay: Duration::from_secs(180),
        }
    }
}

pub type AsyncWork =
    Pin<Box<dyn Future<Output = error_stack::Result<(), ErrorOperation>> + Sync + Send>>;

#[async_trait::async_trait]
pub trait MessageQueue<T>: 'static + Sync + Send
where
    T: 'static + Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    type DatabaseConnection: DatabaseConnection;

    fn new<F>(db: Self::DatabaseConnection, name: &str, config: MQConfig, process: F) -> Self
    where
        F: 'static + Fn(T) -> AsyncWork + Sync + Send;

    fn start_workers(&self);

    async fn queue(&self, info: &QueueInfo<T>) -> error_stack::Result<(), KernelError>;

    async fn get_queued_len(&self) -> error_stack::Result<usize, KernelError>;

    async fn get_delayed_infos(
        &self,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError>;

    async fn get_delayed_info(
        &self,
        id: &Uuid,
    ) -> error_stack::Result<Option<ErroredInfo<T>>, KernelError>;

    async fn get_delayed_len(&self) -> error_stack::Result<usize, KernelError>;

    async fn get_failed_infos(
        &self,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError>;

    async fn get_failed_info(
        &self,
        id: &Uuid,
    ) -> error_stack::Result<Option<ErroredInfo<T>>, KernelError>;

    async fn get_failed_len(&self) -> error_stack::Result<usize, KernelError>;
}
