use crate::transfer::{GetRentFromBookIdDto, GetRentFromIdDto, GetRentFromUserIdDto};
use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{Applier, CommandInfo, DestructEventInfo, RentEvent};
use kernel::interface::query::{
    DependOnRentEventQuery, DependOnRentQuery, RentEventQuery, RentQuery,
};
use kernel::interface::update::{
    DependOnRentEventHandler, DependOnRentModifier, RentEventHandler, RentModifier,
};
use kernel::prelude::entity::{BookId, Rent, UserId};
use kernel::KernelError;

#[async_trait::async_trait]
pub trait HandleRentService: 'static + Sync + Send + DependOnRentEventHandler {
    async fn handle_command(
        &self,
        command: CommandInfo<RentEvent, Rent>,
    ) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        self.rent_event_handler()
            .handle(&mut connection, command)
            .await?;

        connection.commit().await?;

        Ok(())
    }
}

impl<T> HandleRentService for T where T: DependOnRentEventHandler {}

#[async_trait::async_trait]
pub trait GetRentService:
    'static + Sync + Send + DependOnRentQuery + DependOnRentEventQuery + DependOnRentModifier
{
    async fn get_rent_from_book(
        &mut self,
        dto: GetRentFromBookIdDto,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
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
        connection.commit().await?;

        Ok(rents)
    }

    async fn get_rent_from_user(
        &mut self,
        dto: GetRentFromUserIdDto,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
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
        connection.commit().await?;

        Ok(rents)
    }

    async fn get_rent_from_id(
        &mut self,
        dto: GetRentFromIdDto,
    ) -> error_stack::Result<Option<Rent>, KernelError> {
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
        connection.commit().await?;

        Ok(rents)
    }
}

impl<T> GetRentService for T where
    T: DependOnRentQuery + DependOnRentEventQuery + DependOnRentModifier
{
}
