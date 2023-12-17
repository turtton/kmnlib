mod book;
mod rent;
mod user;

pub use self::{book::*, rent::*, user::*};
use crate::env;
use crate::error::DriverError;
use error_stack::{Report, ResultExt};
use kernel::interface::database::QueryDatabaseConnection;
use sqlx::pool::PoolConnection;
use sqlx::{Pool, Postgres};

static POSTGRES_URL: &str = "POSTGRES_URL";

pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    async fn new() -> Result<Self, Report<DriverError>> {
        let url = env(POSTGRES_URL)?;
        let pool = Pool::connect(&url)
            .await
            .change_context_lazy(|| DriverError::SqlX)?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl QueryDatabaseConnection<PoolConnection<Postgres>> for PostgresDatabase {
    type Error = DriverError;
    async fn transact(&self) -> Result<PoolConnection<Postgres>, Report<DriverError>> {
        let con = self
            .pool
            .acquire()
            .await
            .change_context_lazy(|| DriverError::SqlX)?;
        Ok(con)
    }
}
