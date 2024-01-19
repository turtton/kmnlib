use crate::KernelError;

#[async_trait::async_trait]
pub trait QueryDatabaseConnection<Connection>: 'static + Sync + Send {
    async fn transact(&self) -> error_stack::Result<Connection, KernelError>;
}

pub trait DependOnDatabaseConnection<Connection>: 'static + Sync + Send {
    type DatabaseConnection: QueryDatabaseConnection<Connection>;
    fn database_connection(&self) -> &Self::DatabaseConnection;
}

impl<T, C> DependOnDatabaseConnection<C> for T
where
    T: QueryDatabaseConnection<C>,
{
    type DatabaseConnection = T;
    fn database_connection(&self) -> &Self::DatabaseConnection {
        self
    }
}
