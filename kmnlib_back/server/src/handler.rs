use driver::database::PostgresDatabase;
use kernel::KernelError;
use std::ops::Deref;
use std::sync::Arc;
use vodca::References;

#[derive(Clone)]
pub struct AppModule(Arc<Handler>);

impl AppModule {
    pub async fn new() -> error_stack::Result<Self, KernelError> {
        Ok(Self(Arc::new(Handler::init().await?)))
    }
}

impl Deref for AppModule {
    type Target = Handler;
    fn deref(&self) -> &Self::Target {
        Deref::deref(&self.0)
    }
}

#[derive(References)]
pub struct Handler {
    pgpool: PostgresDatabase,
}

impl Handler {
    pub async fn init() -> error_stack::Result<Self, KernelError> {
        let pgpool = PostgresDatabase::new().await?;

        Ok(Self { pgpool })
    }
}
