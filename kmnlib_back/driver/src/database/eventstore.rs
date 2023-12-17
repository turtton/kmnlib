mod book;
mod user;

pub use self::{book::*, user::*};

use crate::env;
use crate::error::DriverError;
use error_stack::{Context, Report, ResultExt};
use eventstore::{
    AppendToStreamOptions, Client, ClientSettings, EventData, ReadStreamOptions, RecordedEvent,
    ResolvedEvent, StreamPosition,
};
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
    id: Option<impl AsRef<Uuid>>,
    rev_version: Option<EventVersion<T>>,
    event: impl serde::Serialize,
) -> Result<EventVersion<T>, Report<DriverError>> {
    let expected_rev = rev_version.map_or(Ok(eventstore::ExpectedRevision::Any), |version| {
        u64::try_from(*version.as_ref())
            .map(eventstore::ExpectedRevision::Exact)
            .change_context_lazy(|| DriverError::Conversion)
            .attach_version(version.as_ref().to_string())
    })?;
    let option = AppendToStreamOptions::default().expected_revision(expected_rev);
    let event = EventData::json(&event_type, &event)
        .change_context_lazy(|| DriverError::EventStore)
        .attach_json(&event_type)?;
    let string_id = id.map(|uuid| uuid.as_ref().to_string());

    let result = client
        .append_to_stream(
            create_stream_name(stream_name, string_id.as_deref()),
            &option,
            event,
        )
        .await
        .change_context_lazy(|| DriverError::EventStore)?;

    let raw_version = i64::try_from(result.next_expected_version)
        .change_context_lazy(|| DriverError::Conversion)
        .attach_version(result.next_expected_version.to_string())?;
    let next_version = EventVersion::new(raw_version);
    Ok(next_version)
}

pub async fn read_stream<T>(
    client: &Client,
    stream_name: &str,
    id: Option<impl AsRef<Uuid>>,
    version: Option<EventVersion<T>>,
) -> Result<Vec<RecordedEvent>, Report<DriverError>> {
    let string_id = id.map(|uuid| uuid.as_ref().to_string());
    let stream_name = create_stream_name(stream_name, string_id.as_deref());
    let option = ReadStreamOptions::default();
    let option = match version {
        Some(version) => option.position(
            u64::try_from(*version.as_ref())
                .map(StreamPosition::Position)
                .change_context_lazy(|| DriverError::Conversion)
                .attach_version(version.as_ref().to_string())?,
        ),
        None => option,
    };
    let mut result = client
        .read_stream(stream_name, &option)
        .await
        .change_context_lazy(|| DriverError::EventStore)?;
    let mut events = Vec::new();
    loop {
        match result.next().await {
            Ok(Some(ResolvedEvent { event: Some(e), .. })) => events.push(e),
            Ok(None) => break,
            Err(error) => Err(error).change_context_lazy(|| DriverError::EventStore)?,
            _ => {}
        }
    }
    Ok(events)
}

fn create_stream_name(name: &str, id: Option<&str>) -> String {
    match id {
        None => name.to_string(),
        Some(id) => format!("{name}_{id}"),
    }
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
