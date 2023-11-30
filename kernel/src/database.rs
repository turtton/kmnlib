use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait DatabaseConnection<Connection>: 'static + Sync + Send {
    type Error: Context;
    async fn transact(&self) -> Result<Connection, Report<Self::Error>>;
}

pub trait DependOnDatabaseConnection<Connection>: 'static + Sync + Send {
    type DatabaseConnection: DatabaseConnection<Connection>;
    fn database_connection(&self) -> &Self::DatabaseConnection;
}

impl<T, C> DependOnDatabaseConnection<C> for T
where
    T: DatabaseConnection<C>,
{
    type DatabaseConnection = T;
    fn database_connection(&self) -> &Self::DatabaseConnection {
        self
    }
}
