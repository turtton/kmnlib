use crate::database::eventstore::append_event;
use crate::error::DriverError;
use error_stack::Report;
use eventstore::Client;
use kernel::interface::command::{UserCommand, UserCommandHandler, BOOK_STREAM_NAME};
use kernel::prelude::entity::{EventVersion, User, UserId, UserName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum UserEvent {
    Created { name: UserName },
    UpdatedUserName { name: UserName },
    DeletedUser,
}

impl UserEvent {
    fn convert(command: UserCommand) -> (String, UserId, Option<EventVersion<User>>, Self) {
        match command {
            UserCommand::Create { id, name } => {
                let event = Self::Created { name };
                ("created-user".to_string(), id, None, event)
            }
            UserCommand::UpdateName { id, name } => {
                let event = Self::UpdatedUserName { name };
                ("updated-user".to_string(), id, None, event)
            }
            UserCommand::Delete { id } => {
                let event = Self::DeletedUser;
                ("deleted-user".to_string(), id, None, event)
            }
        }
    }
}

pub struct EventStoreUserCommandHolder {
    client: Client,
}

impl EventStoreUserCommandHolder {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl UserCommandHandler for EventStoreUserCommandHolder {
    type Error = DriverError;
    async fn handle(
        &self,
        command: UserCommand,
    ) -> Result<EventVersion<User>, Report<Self::Error>> {
        let (event_type, id, _, event) = UserEvent::convert(command);
        append_event(&self.client, BOOK_STREAM_NAME, event_type, id, None, event).await
    }
}

#[cfg(test)]
mod test {
    use crate::database::eventstore::{create_event_store_client, EventStoreUserCommandHolder};
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::command::{UserCommand, UserCommandHandler};
    use kernel::prelude::entity::{UserId, UserName};
    use uuid::Uuid;

    #[test_with::env(EVENTSTORE)]
    #[tokio::test]
    async fn handle() -> Result<(), Report<DriverError>> {
        let client = create_event_store_client()?;
        let handler = EventStoreUserCommandHolder::new(client);
        let id = UserId::new(Uuid::new_v4());
        let create_user = UserCommand::Create {
            id: id.clone(),
            name: UserName::new("test".to_string()),
        };
        handler.handle(create_user).await?;
        let update_user = UserCommand::UpdateName {
            id: id.clone(),
            name: UserName::new("test2".to_string()),
        };
        handler.handle(update_user).await?;
        let delete_user = UserCommand::Delete { id: id.clone() };
        handler.handle(delete_user).await?;
        Ok(())
    }
}
