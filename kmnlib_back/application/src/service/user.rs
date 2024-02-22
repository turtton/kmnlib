use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{Applier, CommandInfo, UserEvent};
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::interface::update::{
    DependOnUserEventHandler, DependOnUserModifier, UserEventHandler, UserModifier,
};
use kernel::prelude::entity::{User, UserId};
use kernel::KernelError;

use crate::transfer::{GetAllUserDto, GetUserDto};

#[async_trait::async_trait]
pub trait HandleUserService: 'static + Sync + Send + DependOnUserEventHandler {
    async fn handle_event(&self, event: UserEvent) -> error_stack::Result<UserId, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let command = CommandInfo::new(event, None);
        let id = self
            .user_event_handler()
            .handle(&mut connection, command)
            .await?;

        connection.commit().await?;

        Ok(id)
    }
}

impl<T> HandleUserService for T where T: DependOnUserEventHandler {}

#[async_trait::async_trait]
pub trait GetUserService:
    'static + Sync + Send + DependOnUserQuery + DependOnUserModifier + DependOnUserEventQuery
{
    async fn get_all(
        &self,
        GetAllUserDto { limit, offset }: GetAllUserDto,
    ) -> error_stack::Result<Vec<User>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let mut users = self
            .user_query()
            .get_all(&mut connection, &limit, &offset)
            .await?;

        for user in &mut users {
            let events = self
                .user_event_query()
                .get_events(&mut connection, user.id(), Some(user.version()))
                .await?;
            if !events.is_empty() {
                events.into_iter().for_each(|e| user.apply(e));
                self.user_modifier().update(&mut connection, user).await?;
            }
        }

        connection.commit().await?;

        Ok(users)
    }
    async fn get_user(&self, dto: GetUserDto) -> error_stack::Result<Option<User>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = dto.id;
        let id = UserId::new(id);
        let mut user = self.user_query().find_by_id(&mut connection, &id).await?;
        let user_exists = user.is_some();

        let version = user.as_ref().map(|u| u.version());
        let user_events = self
            .user_event_query()
            .get_events(&mut connection, &id, version)
            .await?;

        user_events.into_iter().for_each(|event| {
            user.apply(event);
        });

        match (user_exists, &user) {
            (false, Some(user)) => self.user_modifier().create(&mut connection, user).await?,
            (true, Some(user)) => self.user_modifier().update(&mut connection, user).await?,
            (true, None) => self.user_modifier().delete(&mut connection, &id).await?, // Not reachable
            (false, None) => (),
        }
        connection.commit().await?;

        Ok(user)
    }
}

impl<T> GetUserService for T where
    T: DependOnUserQuery + DependOnUserModifier + DependOnUserEventQuery
{
}
