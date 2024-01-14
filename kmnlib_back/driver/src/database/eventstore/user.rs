use eventstore::Client;

use kernel::interface::command::{UserCommand, UserCommandHandler, USER_STREAM_NAME};
use kernel::interface::event::UserEvent;
use kernel::interface::query::UserEventQuery;
use kernel::prelude::entity::{EventVersion, User, UserId};

use crate::database::eventstore::{append_event, read_stream};
use crate::error::DriverError;

pub struct EventStoreUserHandler {
    client: Client,
}

impl EventStoreUserHandler {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl UserCommandHandler for EventStoreUserHandler {
    type Error = DriverError;
    async fn handle(&self, command: UserCommand) -> Result<EventVersion<User>, Self::Error> {
        let (event_type, id, _, event) = UserEvent::convert(command);
        append_event(
            &self.client,
            USER_STREAM_NAME,
            event_type,
            Some(id),
            None,
            event,
        )
        .await
    }
}

#[async_trait::async_trait]
impl UserEventQuery for EventStoreUserHandler {
    type Error = DriverError;
    async fn get_events(
        &self,
        id: &UserId,
        since: Option<EventVersion<User>>,
    ) -> Result<Vec<UserEvent>, Self::Error> {
        read_stream(&self.client, USER_STREAM_NAME, Some(id), since)
            .await?
            .iter()
            .map(|event| event.as_json::<UserEvent>())
            .collect::<serde_json::Result<Vec<UserEvent>>>()
            .map_err(DriverError::from)
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use kernel::interface::command::{UserCommand, UserCommandHandler};
    use kernel::interface::event::UserEvent;
    use kernel::interface::query::UserEventQuery;
    use kernel::prelude::entity::{UserId, UserName, UserRentLimit};

    use crate::database::eventstore::{create_event_store_client, EventStoreUserHandler};
    use crate::error::DriverError;

    #[test_with::env(EVENTSTORE_TEST)]
    #[tokio::test]
    async fn test() -> Result<(), DriverError> {
        let client = create_event_store_client()?;
        let handler = EventStoreUserHandler::new(client);
        let id = UserId::new(Uuid::new_v4());

        let create_user = UserCommand::Create {
            id: id.clone(),
            name: UserName::new("test".to_string()),
            rent_limit: UserRentLimit::new(1),
        };
        handler.handle(create_user.clone()).await?;

        let mut expected = vec![UserEvent::convert(create_user).3];
        assert_eq!(handler.get_events(&id, None).await?, expected);

        let update_user = UserCommand::Update {
            id: id.clone(),
            name: Some(UserName::new("test2".to_string())),
            rent_limit: None,
        };
        handler.handle(update_user.clone()).await?;

        expected.push(UserEvent::convert(update_user).3);
        assert_eq!(handler.get_events(&id, None).await?, expected);

        let delete_user = UserCommand::Delete { id: id.clone() };
        handler.handle(delete_user.clone()).await?;

        expected.push(UserEvent::convert(delete_user).3);
        assert_eq!(handler.get_events(&id, None).await?, expected);
        Ok(())
    }
}
