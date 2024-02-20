use crate::transfer::{
    CreateRentDto, GetRentFromBookIdDto, GetRentFromIdDto, GetRentFromUserIdDto, RentDto,
};
use error_stack::Report;
use kernel::interface::database::{
    DependOnDatabaseConnection, QueryDatabaseConnection, Transaction,
};
use kernel::interface::event::{Applier, CommandInfo, DestructEventInfo, RentEvent};
use kernel::interface::query::{
    DependOnRentEventQuery, DependOnRentQuery, RentEventQuery, RentQuery,
};
use kernel::interface::update::{
    DependOnRentEventHandler, DependOnRentModifier, RentEventHandler, RentModifier,
};
use kernel::prelude::entity::{BookId, EventVersion, Rent, UserId};
use kernel::KernelError;

#[async_trait::async_trait]
pub trait GetRentService<Connection: Transaction>:
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

        for event in rent_events {
            let DestructEventInfo { event, version, .. } = event.into_destruct();
            match event {
                RentEvent::Rent { book_id, user_id } => {
                    let rent = Rent::new(version, book_id, user_id);
                    self.rent_modifier().create(&mut connection, &rent).await?;
                    rents.push(rent);
                }
                RentEvent::Return { book_id, user_id } => {
                    let target_index = rents
                        .iter()
                        .position(|rent| rent.book_id() == &book_id && rent.user_id() == &user_id);
                    match target_index {
                        None => (),
                        Some(index) => {
                            rents.remove(index);
                            self.rent_modifier()
                                .delete(&mut connection, &book_id, &user_id)
                                .await?;
                        }
                    }
                }
            }
        }

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

        for event in rent_events {
            let DestructEventInfo { event, version, .. } = event.into_destruct();
            match event {
                RentEvent::Rent { book_id, user_id } => {
                    let rent = Rent::new(version, book_id, user_id);
                    self.rent_modifier().create(&mut connection, &rent).await?;
                    rents.push(rent);
                }
                RentEvent::Return { book_id, user_id } => {
                    let target_index = rents
                        .iter()
                        .position(|rent| rent.book_id() == &book_id && rent.user_id() == &user_id);
                    match target_index {
                        None => (),
                        Some(index) => {
                            rents.remove(index);
                            self.rent_modifier()
                                .delete(&mut connection, &book_id, &user_id)
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(rents
            .into_iter()
            .map(RentDto::try_from)
            .collect::<Result<Vec<RentDto>, Report<KernelError>>>()?)
    }

    async fn get_rent_from_id(
        &mut self,
        dto: GetRentFromIdDto,
    ) -> error_stack::Result<Option<RentDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let book_id = BookId::new(dto.book_id);
        let user_id = UserId::new(dto.user_id);
        let mut rents = self
            .rent_query()
            .find_by_id(&mut connection, &book_id, &user_id)
            .await?;
        let rent_exists = rents.is_some();

        let version = rents.as_ref().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events(&mut connection, &book_id, &user_id, version)
            .await?;

        rent_events.into_iter().for_each(|event| {
            rents.apply(event);
        });

        match (rent_exists, &rents) {
            (false, Some(rent)) => self.rent_modifier().create(&mut connection, rent).await?,
            (true, Some(_)) => (),
            (true, None) => {
                self.rent_modifier()
                    .delete(&mut connection, &book_id, &user_id)
                    .await?
            }
            (false, None) => (),
        }

        Ok(rents.map(RentDto::try_from).transpose()?)
    }
}

impl<Connection: Transaction, T> GetRentService<Connection> for T where
    T: DependOnDatabaseConnection<Connection>
        + DependOnRentQuery<Connection>
        + DependOnRentEventQuery<Connection>
        + DependOnRentModifier<Connection>
{
}

#[async_trait::async_trait]
pub trait RentService<Connection: Transaction>:
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

impl<Connection: Transaction, T> RentService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnRentEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait ReturnService<Connection: Transaction>:
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

impl<Connection: Transaction, T> ReturnService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnRentEventHandler<Connection>
{
}
