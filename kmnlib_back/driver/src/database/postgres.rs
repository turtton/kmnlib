use sqlx::pool::PoolConnection;
use sqlx::{Pool, Postgres};

use kernel::interface::database::QueryDatabaseConnection;

use crate::env;
use crate::error::DriverError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

static POSTGRES_URL: &str = "POSTGRES_URL";

pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    async fn new() -> Result<Self, DriverError> {
        let url = env(POSTGRES_URL)?;
        let pool = Pool::connect(&url).await?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl QueryDatabaseConnection<PoolConnection<Postgres>> for PostgresDatabase {
    type Error = DriverError;
    async fn transact(&self) -> Result<PoolConnection<Postgres>, DriverError> {
        let con = self.pool.acquire().await?;
        Ok(con)
    }
}
