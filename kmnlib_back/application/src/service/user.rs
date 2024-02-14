use uuid::Uuid;

use kernel::interface::database::{
    DependOnDatabaseConnection, QueryDatabaseConnection, Transaction,
};
use kernel::interface::event::{Applier, CommandInfo, UserEvent};
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::interface::update::{DependOnUserEventHandler, UserEventHandler};
use kernel::prelude::entity::{UserId, UserName, UserRentLimit};
use kernel::KernelError;

use crate::transfer::{CreateUserDto, DeleteUserDto, GetUserDto, UpdateUserDto, UserDto};

#[async_trait::async_trait]
pub trait GetUserService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserQuery<Connection>
    + DependOnUserEventQuery<Connection>
{
    async fn get_user(
        &mut self,
        dto: GetUserDto,
    ) -> error_stack::Result<Option<UserDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = dto.id;
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

impl<Connection: Transaction + Send, T> GetUserService<Connection> for T where
    T: DependOnDatabaseConnection<Connection>
        + DependOnUserQuery<Connection>
        + DependOnUserEventQuery<Connection>
{
}

#[async_trait::async_trait]
pub trait CreateUserService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserEventHandler<Connection>
{
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

        Ok(uuid)
    }
}

impl<Connection: Transaction + Send, T> CreateUserService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnUserEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait UpdateUserService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserEventHandler<Connection>
{
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

        Ok(())
    }
}

impl<Connection: Transaction + Send, T> UpdateUserService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnUserEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait DeleteUserService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserEventHandler<Connection>
{
    async fn delete_user(&mut self, dto: DeleteUserDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = UserId::new(dto.id);
        let user = UserEvent::Delete { id };
        let command = CommandInfo::new(user, None);

        self.user_event_handler()
            .handle(&mut connection, command)
            .await?;

        Ok(())
    }
}

impl<Connection: Transaction + Send, T> DeleteUserService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnUserEventHandler<Connection>
{
}
