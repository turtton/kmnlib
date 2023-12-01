use crate::database::eventstore::append_event;
use crate::error::DriverError;
use error_stack::Report;
use eventstore::Client;
use kernel::interface::command::{BookCommand, BookCommandHandler, USER_STREAM_NAME};
use kernel::prelude::entity::{Book, BookId, EventVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum BookEvent {
    Created { title: String },
    Rented,
    Returned,
    Deleted,
}

impl BookEvent {
    fn convert(command: BookCommand) -> (String, BookId, Option<EventVersion<Book>>, Self) {
        match command {
            BookCommand::Create { id, title } => {
                let event = Self::Created {
                    title: title.as_ref().clone(),
                };
                ("created-book".to_string(), id, None, event)
            }
            BookCommand::Rent { id, rev_version } => {
                let event = Self::Rented;
                ("rented-book".to_string(), id, Some(rev_version), event)
            }
            BookCommand::Return { id, rev_version } => {
                let event = Self::Returned;
                ("returned-book".to_string(), id, Some(rev_version), event)
            }
            BookCommand::Delete { id } => {
                let event = Self::Deleted;
                ("deleted-book".to_string(), id, None, event)
            }
        }
    }
}

pub struct EventStoreBookCommandHandler {
    client: Client,
}

impl EventStoreBookCommandHandler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl BookCommandHandler for EventStoreBookCommandHandler {
    type Error = DriverError;
    async fn handle(
        &self,
        command: BookCommand,
    ) -> Result<EventVersion<Book>, Report<DriverError>> {
        let (event_type, id, rev_version, event) = BookEvent::convert(command);
        append_event(
            &self.client,
            USER_STREAM_NAME,
            event_type,
            id,
            rev_version,
            event,
        )
        .await
    }
}

#[cfg(test)]
mod test {
    use crate::database::eventstore::{create_event_store_client, EventStoreBookCommandHandler};
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::command::{BookCommand, BookCommandHandler};
    use kernel::prelude::entity::{BookId, BookTitle};
    use uuid::Uuid;

    #[test_with::env(EVENTSTORE)]
    #[tokio::test]
    async fn handle() -> Result<(), Report<DriverError>> {
        let client = create_event_store_client()?;
        let handler = EventStoreBookCommandHandler::new(client);
        let id = BookId::new(Uuid::new_v4());
        let create_book = BookCommand::Create {
            id: id.clone(),
            title: BookTitle::new("test".to_string()),
        };
        let created_next = handler.handle(create_book).await?;
        let rent_book = BookCommand::Rent {
            id: id.clone(),
            rev_version: created_next,
        };
        let rented_next = handler.handle(rent_book).await?;
        let return_book = BookCommand::Return {
            id: id.clone(),
            rev_version: rented_next,
        };
        let _ = handler.handle(return_book).await?;
        let delete_book = BookCommand::Delete { id: id.clone() };
        handler.handle(delete_book).await?;
        Ok(())
    }
}
