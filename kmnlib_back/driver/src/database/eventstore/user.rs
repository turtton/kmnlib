use error_stack::ResultExt;
use eventstore::Client;

use kernel::interface::command::{UserCommand, UserCommandHandler, USER_STREAM_NAME};
use kernel::interface::event::UserEvent;
use kernel::interface::query::UserEventQuery;
use kernel::prelude::entity::{EventVersion, User, UserId};
use kernel::KernelError;

use crate::database::eventstore::{append_event, read_stream};

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
    async fn handle(
        &self,
        command: UserCommand,
    ) -> error_stack::Result<EventVersion<User>, KernelError> {
        let (event_type, id, _, event) = UserEvent::convert(command);
        append_event(
            &self.client,
            USER_STREAM_NAME,
            event_type,
            Some(&id.as_ref().to_string()),
            None,
            event,
        )
        .await
    }
}

#[async_trait::async_trait]
impl UserEventQuery for EventStoreUserHandler {
    async fn get_events(
        &self,
        id: &UserId,
        since: Option<&EventVersion<User>>,
    ) -> error_stack::Result<Vec<UserEvent>, KernelError> {
        read_stream(
            &self.client,
            USER_STREAM_NAME,
            Some(&id.as_ref().to_string()),
            since,
        )
        .await?
        .iter()
        .map(|event| event.as_json::<UserEvent>())
        .collect::<serde_json::Result<Vec<UserEvent>>>()
        .change_context_lazy(|| KernelError::Internal)
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use kernel::interface::command::{UserCommand, UserCommandHandler};
    use kernel::interface::event::UserEvent;
    use kernel::interface::query::UserEventQuery;
    use kernel::prelude::entity::{UserId, UserName, UserRentLimit};
    use kernel::KernelError;

    use crate::database::eventstore::{create_event_store_client, EventStoreUserHandler};

    #[test_with::env(EVENTSTORE_TEST)]
    #[tokio::test]
    async fn basic_modification() -> error_stack::Result<(), KernelError> {
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
