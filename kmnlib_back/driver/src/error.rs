use std::num::TryFromIntError;

use eventstore::ClientSettingsParseError;

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error(transparent)]
    SqlX(sqlx::Error),
    #[error(transparent)]
    EventStore(eventstore::Error),
    #[error(transparent)]
    Env(dotenvy::Error),
    #[error(transparent)]
    Conversion(anyhow::Error),
    #[error(transparent)]
    Serde(serde_json::Error),
}

impl From<sqlx::Error> for DriverError {
    fn from(value: sqlx::Error) -> Self {
        Self::SqlX(value)
    }
}

impl From<eventstore::Error> for DriverError {
    fn from(value: eventstore::Error) -> Self {
        Self::EventStore(value)
    }
}

impl From<ClientSettingsParseError> for DriverError {
    fn from(value: ClientSettingsParseError) -> Self {
        Self::Conversion(anyhow::Error::new(value))
    }
}

impl From<dotenvy::Error> for DriverError {
    fn from(value: dotenvy::Error) -> Self {
        Self::Env(value)
    }
}

impl From<serde_json::Error> for DriverError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<TryFromIntError> for DriverError {
    fn from(value: TryFromIntError) -> Self {
        Self::Conversion(anyhow::Error::new(value))
    }
}
