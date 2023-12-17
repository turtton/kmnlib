use crate::error::DriverError;
use error_stack::{Report, ResultExt};

pub mod database;
pub mod error;

pub(crate) fn env(key: &str) -> Result<String, Report<DriverError>> {
    dotenvy::var(key)
        .change_context_lazy(|| DriverError::Env)
        .attach_printable_lazy(|| format!("Env {} not specified", key))
}
