use error_stack::{Report, ResultExt};
use eventstore::{
    AppendToStreamOptions, Client, ClientSettings, Error, EventData, ReadStreamOptions,
    RecordedEvent, ResolvedEvent, StreamPosition,
};

use kernel::prelude::entity::EventVersion;
use kernel::KernelError;

use crate::env;
use crate::error::ConvertError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

static EVENTSTORE_URL: &str = "EVENTSTORE_URL";

pub fn create_event_store_client() -> error_stack::Result<Client, KernelError> {
    let settings = env(EVENTSTORE_URL)?
        .parse::<ClientSettings>()
        .change_context_lazy(|| KernelError::Internal)?;
    Client::new(settings).change_context_lazy(|| KernelError::Internal)
}

pub async fn append_event<T>(
    client: &Client,
    stream_name: &str,
    event_type: String,
    id_str: Option<&str>,
    rev_version: Option<EventVersion<T>>,
    event: impl serde::Serialize,
) -> error_stack::Result<EventVersion<T>, KernelError> {
    let expected_rev =
        rev_version.map_or(
            Ok(eventstore::ExpectedRevision::Any),
            |version| match version {
                EventVersion::Nothing => Ok(eventstore::ExpectedRevision::NoStream),
                EventVersion::Exact(version, _) => u64::try_from(version)
                    .map(eventstore::ExpectedRevision::Exact)
                    .change_context_lazy(|| KernelError::Internal),
            },
        )?;
    let option = AppendToStreamOptions::default().expected_revision(expected_rev);
    let event =
        EventData::json(&event_type, &event).change_context_lazy(|| KernelError::Internal)?;

    let result = client
        .append_to_stream(create_stream_name(stream_name, id_str), &option, event)
        .await
        .convert_error()?;

    let raw_version = i64::try_from(result.next_expected_version)
        .change_context_lazy(|| KernelError::Internal)?;
    let next_version = EventVersion::new(raw_version);
    Ok(next_version)
}

pub async fn read_stream<T>(
    client: &Client,
    stream_name: &str,
    id_str: Option<&str>,
    version: Option<EventVersion<T>>,
) -> error_stack::Result<Vec<RecordedEvent>, KernelError> {
    let stream_name = create_stream_name(stream_name, id_str);
    let option = ReadStreamOptions::default();
    let option = match version {
        Some(EventVersion::Exact(version, ..)) => option.position(
            u64::try_from(version)
                .map(StreamPosition::Position)
                .change_context_lazy(|| KernelError::Internal)?,
        ),
        _ => option,
    };
    let mut result = client
        .read_stream(stream_name, &option)
        .await
        .convert_error()?;

    let mut events = Vec::new();
    loop {
        let next = result.next().await;
        let next = match next {
            Err(Error::ResourceNotFound) => return Ok(Vec::new()),
            _ => next.convert_error()?,
        };
        match next {
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

impl<T> ConvertError for Result<T, Error> {
    type Ok = T;

    fn convert_error(self) -> error_stack::Result<T, KernelError> {
        self.map_err(|error| match error {
            Error::DeadlineExceeded => Report::from(error).change_context(KernelError::Timeout),
            Error::WrongExpectedVersion { .. } => {
                Report::from(error).change_context(KernelError::Concurrency)
            }
            _ => Report::from(error).change_context(KernelError::Internal),
        })
    }
}
