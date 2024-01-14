use crate::error::DriverError;

pub mod database;
pub mod error;

pub(crate) fn env(key: &str) -> Result<String, DriverError> {
    dotenvy::var(key).map_err(DriverError::from)
}
