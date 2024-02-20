use crate::KernelError;

#[async_trait::async_trait]
pub trait DatabaseConnection: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn transact(&self) -> error_stack::Result<Self::Transaction, KernelError>;
}

pub trait DependOnDatabaseConnection: 'static + Sync + Send {
    type DatabaseConnection: DatabaseConnection;
    fn database_connection(&self) -> &Self::DatabaseConnection;
}

impl<T> DependOnDatabaseConnection for T
where
    T: DatabaseConnection,
{
    type DatabaseConnection = T;
    fn database_connection(&self) -> &Self::DatabaseConnection {
        self
    }
}

#[async_trait::async_trait]
pub trait Transaction: 'static + Sync + Send {
    async fn commit(self) -> error_stack::Result<(), KernelError>;
    async fn roll_back(self) -> error_stack::Result<(), KernelError>;
}
