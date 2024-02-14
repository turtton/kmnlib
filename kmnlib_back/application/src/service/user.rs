use uuid::Uuid;

use kernel::interface::database::{
    DependOnDatabaseConnection, QueryDatabaseConnection, Transaction,
};
use kernel::interface::event::Applier;
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::prelude::entity::UserId;
use kernel::KernelError;

use crate::transfer::UserDto;

#[async_trait::async_trait]
pub trait GetUserService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserQuery<Connection>
    + DependOnUserEventQuery<Connection>
{
    async fn get_user(&mut self, id: Uuid) -> error_stack::Result<Option<UserDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = UserId::new(id);
        let mut user = self.user_query().find_by_id(&mut connection, &id).await?;

        let version = user.as_ref().map(|u| u.version());
        let user_events = self
            .user_event_query()
            .get_events(&mut connection, &id, version)
            .await?;

        user_events.into_iter().for_each(|event| {
            user.apply(event);
        });

        match user {
            None => Ok(None),
            Some(user) => Ok(Some(UserDto::try_from(user)?)),
        }
    }
}
