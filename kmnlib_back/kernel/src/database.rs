use crate::KernelError;

#[async_trait::async_trait]
pub trait QueryDatabaseConnection<Connection: Transaction>: 'static + Sync + Send {
    async fn transact(&self) -> error_stack::Result<Connection, KernelError>;
}

pub trait DependOnDatabaseConnection<Connection: Transaction>: 'static + Sync + Send {
    type DatabaseConnection: QueryDatabaseConnection<Connection>;
    fn database_connection(&self) -> &Self::DatabaseConnection;
}

impl<T, C: Transaction> DependOnDatabaseConnection<C> for T
where
    T: QueryDatabaseConnection<C>,
{
    type DatabaseConnection = T;
    fn database_connection(&self) -> &Self::DatabaseConnection {
        self
    }
}

#[async_trait::async_trait]
pub trait Transaction {
    async fn commit(&mut self) -> error_stack::Result<(), KernelError>;
    async fn roll_back(&mut self) -> error_stack::Result<(), KernelError>;
}
