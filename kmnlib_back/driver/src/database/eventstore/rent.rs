use crate::database::eventstore::{append_event, read_stream};
use crate::error::DriverError;
use eventstore::Client;
use kernel::interface::command::{RentCommand, RentCommandHandler, RENT_STREAM_NAME};
use kernel::interface::event::{EventInfo, RentEvent};
use kernel::interface::query::RentEventQuery;
use kernel::prelude::entity::{EventVersion, Rent};
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
    type Error = DriverError;
    async fn handle(&self, command: RentCommand) -> Result<EventVersion<Rent>, Self::Error> {
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
    type Error = DriverError;
    async fn get_events(
        &self,
        since: Option<EventVersion<Rent>>,
    ) -> Result<Vec<EventInfo<RentEvent, Rent>>, Self::Error> {
        read_stream(&self.client, RENT_STREAM_NAME, None, since)
            .await?
            .iter()
            .map(|event| {
                event
                    .revision
                    .try_into()
                    .map_err(DriverError::from)
                    .and_then(|version: i64| {
                        event
                            .as_json::<RentEvent>()
                            .map(|event| EventInfo::new(event, EventVersion::new(version)))
                            .map_err(DriverError::from)
                    })
            })
            .collect::<Result<Vec<EventInfo<RentEvent, Rent>>, DriverError>>()
    }
}

#[cfg(test)]
mod test {
    use crate::error::DriverError;
    use uuid::Uuid;

    use kernel::interface::command::{RentCommand, RentCommandHandler};
    use kernel::interface::event::RentEvent;
    use kernel::interface::query::RentEventQuery;
    use kernel::prelude::entity::{BookId, EventVersion, UserId};

    use crate::database::eventstore::{create_event_store_client, EventStoreRentHandler};

    #[test_with::env(EVENTSTORE_TEST)]
    #[tokio::test]
    async fn basic_modification() -> Result<(), DriverError> {
        let client = create_event_store_client()?;
        let handler = EventStoreRentHandler::new(client.clone());

        let book_id = BookId::new(Uuid::new_v4());
        let user_id = UserId::new(Uuid::new_v4());

        let rent_command = RentCommand::Rent {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: EventVersion::Nothing,
        };
        let version = handler.handle(rent_command.clone()).await?;

        assert_eq!(version, EventVersion::new(0));

        let events = handler.get_events(None).await?;

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].version(), &EventVersion::new(0));
        assert_eq!(events[0].event(), &RentEvent::convert(rent_command).2);

        let return_command = RentCommand::Return {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: version,
        };
        let version = handler.handle(return_command.clone()).await?;

        assert_eq!(version, EventVersion::new(1));

        let events = handler.get_events(None).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].version(), &EventVersion::new(1));
        assert_eq!(events[1].event(), &RentEvent::convert(return_command).2);

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn will_failed() -> Result<(), DriverError> {
        let client = create_event_store_client()?;
        let handler = EventStoreRentHandler::new(client.clone());

        let book_id = BookId::new(Uuid::new_v4());
        let user_id = UserId::new(Uuid::new_v4());

        let rent_command = RentCommand::Rent {
            user_id: user_id.clone(),
            book_id: book_id.clone(),
            expected_version: EventVersion::Nothing,
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
