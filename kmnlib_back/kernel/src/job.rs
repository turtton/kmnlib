use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::KernelError;
use destructure::Destructure;
use error_stack::Context;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::Future;
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

#[async_trait::async_trait]
pub trait JobQueue: 'static + Sync + Send {
    type DatabaseConnection: DatabaseConnection;
    type Transaction: Transaction;
    async fn queue<T: Serialize + Sync + Send>(
        con: &mut Self::Transaction,
        name: &str,
        info: &QueueInfo<T>,
    ) -> error_stack::Result<(), KernelError>;

    async fn listen<T, F, R>(db: Self::DatabaseConnection, name: String, block: F)
    where
        T: Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
        R: Future<Output = error_stack::Result<(), ErrorOperation>> + Send,
        F: Fn(T) -> R + Sync + Send;

    async fn get_queued_len(
        con: &mut Self::Transaction,
        name: &str,
    ) -> error_stack::Result<usize, KernelError>;

    async fn get_delayed<T: for<'de> Deserialize<'de>>(
        con: &mut Self::Transaction,
        name: &str,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError>;

    async fn get_delayed_len(
        con: &mut Self::Transaction,
        name: &str,
    ) -> error_stack::Result<usize, KernelError>;

    async fn get_failed<T: for<'de> Deserialize<'de>>(
        con: &mut Self::Transaction,
        name: &str,
        size: &i64,
        offset: &i64,
    ) -> error_stack::Result<Vec<ErroredInfo<T>>, KernelError>;

    async fn get_failed_len(
        con: &mut Self::Transaction,
        name: &str,
    ) -> error_stack::Result<usize, KernelError>;
}

pub trait DependOnJobQueue: 'static + Sync + Send + DependOnDatabaseConnection {
    type JobQueue: JobQueue<
        DatabaseConnection = Self::DatabaseConnection,
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;

    fn job_queue(&self) -> &Self::JobQueue;
}
