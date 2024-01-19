use error_stack::Report;
use sqlx::pool::PoolConnection;
use sqlx::{Error, Pool, Postgres};

use kernel::interface::database::QueryDatabaseConnection;
use kernel::KernelError;

use crate::env;
use crate::error::ConvertError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

static POSTGRES_URL: &str = "POSTGRES_URL";

pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    async fn new() -> error_stack::Result<Self, KernelError> {
        let url = env(POSTGRES_URL)?;
        let pool = Pool::connect(&url).await.convert_error()?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl QueryDatabaseConnection<PoolConnection<Postgres>> for PostgresDatabase {
    async fn transact(&self) -> error_stack::Result<PoolConnection<Postgres>, KernelError> {
        let con = self.pool.acquire().await.convert_error()?;
        Ok(con)
    }
}

impl<T> ConvertError for Result<T, Error> {
    type Ok = T;
    fn convert_error(self) -> error_stack::Result<T, KernelError> {
        self.map_err(|error| match error {
            Error::PoolTimedOut => Report::from(error).change_context(KernelError::Timeout),
            _ => Report::from(error).change_context(KernelError::Internal),
        })
    }
}
