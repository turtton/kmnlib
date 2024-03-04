mod mq;

use crate::env;
use crate::error::ConvertError;
use deadpool_redis::redis::RedisError;
use deadpool_redis::{Config, Connection, Pool, PoolError, Runtime};
use error_stack::{Report, ResultExt};
use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::KernelError;
use std::ops::{Deref, DerefMut};

pub use crate::database::redis::mq::*;

const REDIS_URL: &str = "REDIS_URL";

pub struct RedisDatabase {
    pool: Pool,
}

impl RedisDatabase {
    pub fn new() -> error_stack::Result<Self, KernelError> {
        let url = env(REDIS_URL)?;
        let cfg = Config::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .change_context_lazy(|| KernelError::Internal)?;
        Ok(Self { pool })
    }
}

impl Clone for RedisDatabase {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[async_trait::async_trait]
impl DatabaseConnection for RedisDatabase {
    type Transaction = RedisTransaction;
    async fn transact(&self) -> error_stack::Result<Self::Transaction, KernelError> {
        let con: Connection = self.pool.get().await.convert_error()?;
        Ok(RedisTransaction(con))
    }
}

pub struct RedisTransaction(Connection);

#[async_trait::async_trait]
impl Transaction for RedisTransaction {
    async fn commit(mut self) -> error_stack::Result<(), KernelError> {
        Ok(())
    }

    async fn roll_back(mut self) -> error_stack::Result<(), KernelError> {
        Err(Report::new(KernelError::Internal)
            .attach_printable("roll_back is UnImplemented in redis!!"))
    }
}

impl Deref for RedisTransaction {
    type Target = Connection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RedisTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ConvertError for Result<T, PoolError> {
    type Ok = T;
    fn convert_error(self) -> error_stack::Result<T, KernelError> {
        self.map_err(|error| match error {
            PoolError::Timeout(_) => Report::new(error).change_context(KernelError::Timeout),
            _ => Report::new(error).change_context(KernelError::Internal),
        })
    }
}

impl<T> ConvertError for Result<T, RedisError> {
    type Ok = T;
    fn convert_error(self) -> error_stack::Result<T, KernelError> {
        self.map_err(|error| Report::new(error).change_context(KernelError::Internal))
    }
}
