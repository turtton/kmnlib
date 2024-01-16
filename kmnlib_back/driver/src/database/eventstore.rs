use eventstore::{
    AppendToStreamOptions, Client, ClientSettings, EventData, ReadStreamOptions, RecordedEvent,
    ResolvedEvent, StreamPosition,
};

use kernel::prelude::entity::EventVersion;

use crate::env;
use crate::error::DriverError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

static EVENTSTORE_URL: &str = "EVENTSTORE_URL";

pub fn create_event_store_client() -> Result<Client, DriverError> {
    let settings = env(EVENTSTORE_URL)?.parse::<ClientSettings>()?;
    Client::new(settings).map_err(DriverError::from)
}

pub async fn append_event<T>(
    client: &Client,
    stream_name: &str,
    event_type: String,
    id_str: Option<&str>,
    rev_version: Option<EventVersion<T>>,
    event: impl serde::Serialize,
) -> Result<EventVersion<T>, DriverError> {
    let expected_rev =
        rev_version.map_or(
            Ok(eventstore::ExpectedRevision::Any),
            |version| match version {
                EventVersion::Nothing => Ok(eventstore::ExpectedRevision::NoStream),
                EventVersion::Exact(version, _) => u64::try_from(version)
                    .map(eventstore::ExpectedRevision::Exact)
                    .map_err(DriverError::from),
            },
        )?;
    let option = AppendToStreamOptions::default().expected_revision(expected_rev);
    let event = EventData::json(&event_type, &event)?;

    let result = client
        .append_to_stream(create_stream_name(stream_name, id_str), &option, event)
        .await?;

    let raw_version = i64::try_from(result.next_expected_version)?;
    let next_version = EventVersion::new(raw_version);
    Ok(next_version)
}

pub async fn read_stream<T>(
    client: &Client,
    stream_name: &str,
    id_str: Option<&str>,
    version: Option<EventVersion<T>>,
) -> Result<Vec<RecordedEvent>, DriverError> {
    let stream_name = create_stream_name(stream_name, id_str);
    let option = ReadStreamOptions::default();
    let option = match version {
        Some(EventVersion::Exact(version, ..)) => {
            option.position(u64::try_from(version).map(StreamPosition::Position)?)
        }
        _ => option,
    };
    let mut result = client.read_stream(stream_name, &option).await?;
    let mut events = Vec::new();
    loop {
        match result.next().await? {
            Some(ResolvedEvent { event: Some(e), .. }) => events.push(e),
            None => break,
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
