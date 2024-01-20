use error_stack::{Report, ResultExt};
use eventstore::Client;

use kernel::interface::command::{RentCommand, RentCommandHandler, RENT_STREAM_NAME};
use kernel::interface::event::{EventInfo, RentEvent};
use kernel::interface::query::RentEventQuery;
use kernel::prelude::entity::{EventVersion, Rent};
use kernel::KernelError;

use crate::database::eventstore::{append_event, read_stream};

pub struct EventStoreRentHandler {
    client: Client,
}

impl EventStoreRentHandler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl RentCommandHandler for EventStoreRentHandler {
    async fn handle(
        &self,
        command: RentCommand,
    ) -> error_stack::Result<EventVersion<Rent>, KernelError> {
        let (event_type, expected, event) = RentEvent::convert(command);
        append_event(
            &self.client,
            RENT_STREAM_NAME,
            event_type,
            None,
            expected,
            event,
        )
        .await
    }
}

#[async_trait::async_trait]
impl RentEventQuery for EventStoreRentHandler {
    async fn get_events(
        &self,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        read_stream(&self.client, RENT_STREAM_NAME, None, since)
            .await?
            .iter()
            .map(|event| {
                event
                    .revision
                    .try_into()
                    .map_err(|e| Report::from(e).change_context(KernelError::Internal))
                    .and_then(|version: i64| {
                        event
                            .as_json::<RentEvent>()
                            .map(|event| EventInfo::new(event, EventVersion::new(version)))
                            .change_context_lazy(|| KernelError::Internal)
                    })
            })
            .collect::<error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError>>()
    }
}

// These tests are causes concurrency error please run it one by one
#[cfg(test)]
mod test {
    use uuid::Uuid;

    use kernel::interface::command::{RentCommand, RentCommandHandler};
    use kernel::interface::event::RentEvent;
    use kernel::interface::query::RentEventQuery;
    use kernel::prelude::entity::{BookId, EventVersion, UserId};
    use kernel::KernelError;

    use crate::database::eventstore::{create_event_store_client, EventStoreRentHandler};

    #[ignore]
    #[tokio::test]
    async fn basic_modification() -> error_stack::Result<(), KernelError> {
        let client = create_event_store_client()?;
        let handler = EventStoreRentHandler::new(client.clone());

        let book_id = BookId::new(Uuid::new_v4());
        let user_id = UserId::new(Uuid::new_v4());

        let events = handler.get_events(None).await?;

        let expected = events.last().map_or(EventVersion::Nothing, |event| {
            EventVersion::new(*event.version().as_ref())
        });

        let rent_command = RentCommand::Rent {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: expected.clone(),
        };
        let next_version = handler.handle(rent_command.clone()).await?;

        let next_expected = EventVersion::new(expected.as_ref() + 1);
        assert_eq!(next_version, next_expected.clone());

        let events = handler.get_events(Some(&next_expected.clone())).await?;

        assert_eq!(
            events.len(),
            1,
            "Invalid length. From: {}",
            next_expected.as_ref()
        );
        assert_eq!(events[0].version(), &next_expected);
        assert_eq!(events[0].event(), &RentEvent::convert(rent_command).2);

        let return_command = RentCommand::Return {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: next_version,
        };
        let next_version = handler.handle(return_command.clone()).await?;

        let expected = next_expected;
        let next_expected = EventVersion::new(expected.as_ref() + 1);
        assert_eq!(next_version, next_expected);

        let events = handler.get_events(Some(&expected)).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].version(), &next_expected);
        assert_eq!(events[1].event(), &RentEvent::convert(return_command).2);

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn will_failed() -> error_stack::Result<(), KernelError> {
        let client = create_event_store_client()?;
        let handler = EventStoreRentHandler::new(client.clone());

        let book_id = BookId::new(Uuid::new_v4());
        let user_id = UserId::new(Uuid::new_v4());

        let events = handler.get_events(None).await?;

        let expected = events.last().map_or(EventVersion::Nothing, |event| {
            EventVersion::new(*event.version().as_ref())
        });
        let rent_command = RentCommand::Rent {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: expected,
        };
        let _ = handler.handle(rent_command.clone()).await?;

        let return_command = RentCommand::Return {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: EventVersion::Nothing,
        };
        let result = handler.handle(return_command.clone()).await;

        assert!(result.is_err());

        Ok(())
    }
}
