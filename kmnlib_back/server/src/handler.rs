use crate::mq::{init_command_worker, CommandOperation};
use driver::database::{PostgresDatabase, RedisDatabase, RedisMessageQueue};
use kernel::KernelError;
use std::sync::Arc;
use vodca::References;

#[derive(Clone, References)]
pub struct AppModule {
    handler: Arc<Handler>,
    worker: Arc<Worker>,
}

impl AppModule {
    pub async fn new() -> error_stack::Result<Self, KernelError> {
        let handler = Arc::new(Handler::init().await?);
        let worker = Arc::new(Worker::new(&handler));
        Ok(Self { handler, worker })
    }
}

#[derive(References)]
pub struct Handler {
    pgpool: PostgresDatabase,
    redis_pool: RedisDatabase,
}

impl Handler {
    pub async fn init() -> error_stack::Result<Self, KernelError> {
        let pgpool = PostgresDatabase::new().await?;
        let redis_pool = RedisDatabase::new()?;

        Ok(Self { pgpool, redis_pool })
    }
}

#[derive(References)]
pub struct Worker {
    command: RedisMessageQueue<Arc<Handler>, CommandOperation>,
}

impl Worker {
    pub fn new(handler: &Arc<Handler>) -> Self {
        let command = init_command_worker(handler);
        Self { command }
    }
}
