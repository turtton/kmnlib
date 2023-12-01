mod book;
mod user;

pub use self::{book::*, user::*};

use crate::env;
use crate::error::DriverError;
use error_stack::{Context, Report, ResultExt};
use eventstore::{AppendToStreamOptions, Client, ClientSettings, EventData};
use kernel::prelude::entity::EventVersion;
use uuid::Uuid;

static EVENTSTORE_URL: &str = "EVENTSTORE_URL";

pub fn create_event_store_client() -> Result<Client, Report<DriverError>> {
    env(EVENTSTORE_URL)?
        .parse::<ClientSettings>()
        .change_context_lazy(|| DriverError::EventStore)
        .and_then(|url| Client::new(url).change_context_lazy(|| DriverError::EventStore))
}

pub async fn append_event<T>(
    client: &Client,
    stream_name: &str,
    event_type: String,
    id: impl AsRef<Uuid>,
    rev_version: Option<EventVersion<T>>,
    event: impl serde::Serialize,
) -> Result<EventVersion<T>, Report<DriverError>> {
    let expected_rev = rev_version.map_or(Ok(eventstore::ExpectedRevision::Any), |version| {
        u64::try_from(version.as_ref().clone())
            .map(eventstore::ExpectedRevision::Exact)
            .change_context_lazy(|| DriverError::Conversion)
            .attach_version(version.as_ref().to_string())
    })?;
    let option = AppendToStreamOptions::default().expected_revision(expected_rev);
    let event = EventData::json(&event_type, &event)
        .change_context_lazy(|| DriverError::EventStore)
        .attach_json(&event_type)?
        .id(*id.as_ref());
    let result = client
        .append_to_stream(stream_name, &option, event)
        .await
        .change_context_lazy(|| DriverError::EventStore)?;
    let raw_version = i64::try_from(result.next_expected_version)
        .change_context_lazy(|| DriverError::Conversion)
        .attach_version(result.next_expected_version.to_string())?;
    let next_version = EventVersion::new(raw_version);
    Ok(next_version)
}

trait AttachEventSourceError: ResultExt {
    fn attach_event<T: ToString>(self, event_type: &T) -> Result<Self::Ok, Report<Self::Context>>
    where
        Self: Sized,
    {
        self.attach_printable_lazy(|| format!("command: {}", event_type.to_string()))
    }

    fn attach_json<T: ToString>(self, command: &T) -> Result<Self::Ok, Report<Self::Context>>
    where
        Self: Sized,
    {
        self.attach_printable_lazy(|| "failed to parse command to json")
            .attach_event(command)
    }

    fn attach_version(self, version: String) -> Result<Self::Ok, Report<Self::Context>>
    where
        Self: Sized,
    {
        self.attach_printable_lazy(|| format!("failed to convert version {}", version))
    }
}

impl<T, C: Context> AttachEventSourceError for Result<T, Report<C>> {}
