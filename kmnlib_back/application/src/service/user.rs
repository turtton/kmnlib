use uuid::Uuid;

use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{Applier, CommandInfo, UserEvent};
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::interface::update::{
    DependOnUserEventHandler, DependOnUserModifier, UserEventHandler, UserModifier,
};
use kernel::prelude::entity::{User, UserId, UserName, UserRentLimit};
use kernel::KernelError;

use crate::transfer::{CreateUserDto, DeleteUserDto, GetUserDto, UpdateUserDto, UserDto};

#[async_trait::async_trait]
pub trait HandleUserService: 'static + Sync + Send + DependOnUserEventHandler {
    async fn handle_command(
        &self,
        command: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<UserId, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = self
            .user_event_handler()
            .handle(&mut connection, command)
            .await?;

        connection.commit().await?;

        Ok(id)
    }
}

#[async_trait::async_trait]
pub trait GetUserService:
    'static + Sync + Send + DependOnUserQuery + DependOnUserModifier + DependOnUserEventQuery
{
    async fn get_user(
        &mut self,
        dto: GetUserDto,
    ) -> error_stack::Result<Option<UserDto>, KernelError> {
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
            (true, None) => self.user_modifier().delete(&mut connection, &id).await?,
            (false, None) => (),
        }
        connection.commit().await?;

        match user {
            None => Ok(None),
            Some(user) => Ok(Some(UserDto::try_from(user)?)),
        }
    }
}

impl<T> GetUserService for T where
    T: DependOnUserQuery + DependOnUserModifier + DependOnUserEventQuery
{
}

#[async_trait::async_trait]
pub trait CreateUserService: 'static + Sync + Send + DependOnUserEventHandler {
    async fn create_user(&mut self, dto: CreateUserDto) -> error_stack::Result<Uuid, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let uuid = Uuid::new_v4();
        let id = UserId::new(uuid);
        let user = UserEvent::Create {
            id,
            name: UserName::new(dto.name),
            rent_limit: UserRentLimit::new(dto.rent_limit),
        };
        let command = CommandInfo::new(user, None);

        self.user_event_handler()
            .handle(&mut connection, command)
            .await?;
        connection.commit().await?;

        Ok(uuid)
    }
}

impl<T> CreateUserService for T where T: DependOnUserEventHandler {}

#[async_trait::async_trait]
pub trait UpdateUserService: 'static + Sync + Send + DependOnUserEventHandler {
    async fn update_user(&mut self, dto: UpdateUserDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = UserId::new(dto.id);

        let user = UserEvent::Update {
            id,
            name: dto.name.map(UserName::new),
            rent_limit: dto.rent_limit.map(UserRentLimit::new),
        };
        let command = CommandInfo::new(user, None);

        self.user_event_handler()
            .handle(&mut connection, command)
            .await?;
        connection.commit().await?;

        Ok(())
    }
}

impl<T> UpdateUserService for T where T: DependOnUserEventHandler {}

#[async_trait::async_trait]
pub trait DeleteUserService: 'static + Sync + Send + DependOnUserEventHandler {
    async fn delete_user(&mut self, dto: DeleteUserDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = UserId::new(dto.id);
        let user = UserEvent::Delete { id };
        let command = CommandInfo::new(user, None);

        self.user_event_handler()
            .handle(&mut connection, command)
            .await?;
        connection.commit().await?;

        Ok(())
    }
}

impl<T> DeleteUserService for T where T: DependOnUserEventHandler {}
