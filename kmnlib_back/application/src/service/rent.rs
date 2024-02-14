use crate::transfer::{CreateRentDto, GetRentFromBookIdDto, GetRentFromUserIdDto, RentDto};
use error_stack::Report;
use kernel::interface::database::{
    DependOnDatabaseConnection, QueryDatabaseConnection, Transaction,
};
use kernel::interface::event::{Applier, CommandInfo, RentEvent};
use kernel::interface::query::{
    DependOnRentEventQuery, DependOnRentQuery, RentEventQuery, RentQuery,
};
use kernel::interface::update::{DependOnRentEventHandler, DependOnRentModifier, RentEventHandler};
use kernel::prelude::entity::{BookId, EventVersion, UserId};
use kernel::KernelError;

#[async_trait::async_trait]
pub trait GetRentService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnRentQuery<Connection>
    + DependOnRentEventQuery<Connection>
    + DependOnRentModifier<Connection>
{
    async fn get_rent_from_book(
        &mut self,
        dto: GetRentFromBookIdDto,
    ) -> error_stack::Result<Vec<RentDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let book_id = BookId::new(dto.book_id);
        let mut rents = self
            .rent_query()
            .find_by_book_id(&mut connection, &book_id)
            .await?;

        let version = rents.last().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events_from_book(&mut connection, &book_id, version)
            .await?;

        rent_events.into_iter().for_each(|event| {
            rents.apply(event);
        });
        // TODO: Update entity

        Ok(rents
            .into_iter()
            .map(RentDto::try_from)
            .collect::<Result<Vec<RentDto>, Report<KernelError>>>()?)
    }

    async fn get_rent_from_user(
        &mut self,
        dto: GetRentFromUserIdDto,
    ) -> error_stack::Result<Vec<RentDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let user_id = UserId::new(dto.user_id);
        let mut rents = self
            .rent_query()
            .find_by_user_id(&mut connection, &user_id)
            .await?;

        let version = rents.last().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events_from_user(&mut connection, &user_id, version)
            .await?;

        rent_events.into_iter().for_each(|event| {
            rents.apply(event);
        });

        Ok(rents
            .into_iter()
            .map(RentDto::try_from)
            .collect::<Result<Vec<RentDto>, Report<KernelError>>>()?)
    }
}

impl<Connection: Transaction + Send, T> GetRentService<Connection> for T where
    T: DependOnDatabaseConnection<Connection>
        + DependOnRentQuery<Connection>
        + DependOnRentEventQuery<Connection>
        + DependOnRentModifier<Connection>
{
}

#[async_trait::async_trait]
pub trait RentService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnRentEventHandler<Connection>
{
    async fn rent_book(&mut self, dto: CreateRentDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let version = EventVersion::new(dto.version);
        let book_id = BookId::new(dto.book_id);
        let user_id = UserId::new(dto.user_id);
        let rent = RentEvent::Rent {
            book_id: book_id.clone(),
            user_id: user_id.clone(),
        };
        let command = CommandInfo::new(rent, Some(version));
        self.rent_event_handler()
            .handle(&mut connection, command)
            .await?;

        Ok(())
    }
}

impl<Connection: Transaction + Send, T> RentService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnRentEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait ReturnService<Connection: Transaction + Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnRentEventHandler<Connection>
{
    async fn return_book(&mut self, dto: CreateRentDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let version = EventVersion::new(dto.version);
        let book_id = BookId::new(dto.book_id);
        let user_id = UserId::new(dto.user_id);
        let rent = RentEvent::Return { book_id, user_id };
        let command = CommandInfo::new(rent, Some(version));
        self.rent_event_handler()
            .handle(&mut connection, command)
            .await?;

        Ok(())
    }
}

impl<Connection: Transaction + Send, T> ReturnService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnRentEventHandler<Connection>
{
}
