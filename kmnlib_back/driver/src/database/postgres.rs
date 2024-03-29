use std::ops::{Deref, DerefMut};

use error_stack::Report;
use sqlx::{Error, PgConnection, Pool, Postgres};

use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::KernelError;

use crate::env;
use crate::error::ConvertError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

static POSTGRES_URL: &str = "POSTGRES_URL";

#[derive(Debug, Clone)]
pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    pub async fn new() -> error_stack::Result<Self, KernelError> {
        let url = env(POSTGRES_URL)?;
        let pool = Pool::connect(&url).await.convert_error()?;
        Ok(Self { pool })
    }
}

pub struct PostgresTransaction(sqlx::Transaction<'static, Postgres>);

#[async_trait::async_trait]
impl Transaction for PostgresTransaction {
    async fn commit(self) -> error_stack::Result<(), KernelError> {
        self.0.commit().await.convert_error()
    }

    async fn roll_back(self) -> error_stack::Result<(), KernelError> {
        self.0.rollback().await.convert_error()
    }
}

impl Deref for PostgresTransaction {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PostgresTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl DatabaseConnection for PostgresDatabase {
    type Transaction = PostgresTransaction;
    async fn transact(&self) -> error_stack::Result<PostgresTransaction, KernelError> {
        let con = self.pool.begin().await.convert_error()?;
        Ok(PostgresTransaction(con))
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
