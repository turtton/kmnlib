use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::KernelError;
use error_stack::Context;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::Future;

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
pub trait JobQueue: 'static + Sync + Send {
    type DatabaseConnection: DatabaseConnection;
    type Transaction: Transaction;
    async fn queue<T: Serialize + Sync + Send>(
        con: &mut Self::Transaction,
        name: String,
        data: T,
    ) -> error_stack::Result<(), KernelError>;

    async fn listen<T, F, R>(db: Self::DatabaseConnection, name: String, block: F)
    where
        T: Clone + Serialize + for<'de> Deserialize<'de> + Sync + Send,
        R: Future<Output = error_stack::Result<(), ErrorOperation>> + Send,
        F: Fn(T) -> R + Sync + Send;
}

pub trait DependOnJobQueue: 'static + Sync + Send + DependOnDatabaseConnection {
    type JobQueue: JobQueue<
        DatabaseConnection = Self::DatabaseConnection,
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;

    fn job_queue(&self) -> &Self::JobQueue;
}
