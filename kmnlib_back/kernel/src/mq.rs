mod config;
mod handler;
mod info;

use crate::database::DatabaseConnection;
pub use crate::mq::{config::*, handler::*, info::*};
use crate::KernelError;
use error_stack::Context;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

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

#[async_trait::async_trait]
pub trait MessageQueue<M, T>: 'static + Sync + Send
where
    M: 'static + Clone + Sync + Send,
    T: 'static + Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
{
    type DatabaseConnection: DatabaseConnection;

    fn new<H>(
        db: Self::DatabaseConnection,
        module: M,
        name: &str,
        config: MQConfig,
        process: H,
    ) -> Self
    where
        H: Handler<M, T>;

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
