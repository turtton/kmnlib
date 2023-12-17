use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait QueryDatabaseConnection<Connection>: 'static + Sync + Send {
    type Error: Context;
    async fn transact(&self) -> Result<Connection, Report<Self::Error>>;
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
