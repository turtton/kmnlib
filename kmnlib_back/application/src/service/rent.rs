use crate::service::{GetBookService, GetUserService};
use crate::transfer::{
    GetBookDto, GetRentFromBookIdDto, GetRentFromIdDto, GetRentFromUserIdDto, GetUserDto,
};
use error_stack::Report;
use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{CommandInfo, DestructEventInfo, EventInfo, RentEvent};
use kernel::interface::query::{
    DependOnRentEventQuery, DependOnRentQuery, RentEventQuery, RentQuery,
};
use kernel::interface::update::{
    DependOnRentEventHandler, DependOnRentModifier, RentEventHandler, RentModifier,
};
use kernel::prelude::entity::{EventVersion, ExpectedEventVersion, Rent, ReturnedAt};
use kernel::KernelError;

#[async_trait::async_trait]
pub trait HandleRentService:
    'static + Sync + Send + DependOnRentEventHandler + GetRentService + GetUserService + GetBookService
{
    async fn handle_event(&self, event: RentEvent) -> error_stack::Result<(), KernelError> {
        let command =
            match event {
                RentEvent::Rent { book_id, user_id } => {
                    let dto = GetRentFromIdDto { book_id, user_id };
                    let rents = self.get_rents_from_id(&dto).await?;
                    let rent = rents.last();
                    if rent.is_some() && rent.unwrap().returned_at().is_some() {
                        return Err(Report::new(KernelError::Concurrency).attach_printable(
                            format!(
                                "Target Book({:?}) already rented. User:{:?}",
                                dto.book_id, dto.user_id
                            ),
                        ));
                    }
                    let book_id_dto = GetBookDto { id: dto.book_id };
                    let book = self.get_book(&book_id_dto).await?;
                    if book.is_none() {
                        return Err(Report::new(KernelError::Concurrency).attach_printable(
                            format!("Target Book({:?}) does not exists", book_id_dto.id),
                        ));
                    }
                    let book = book.unwrap();
                    let book_id_dto = GetRentFromBookIdDto {
                        book_id: book_id_dto.id,
                    };
                    let book_rents = self.get_rent_from_book(&book_id_dto).await?;
                    if book_rents.len() >= *book.amount().as_ref() as usize {
                        return Err(Report::new(KernelError::Concurrency).attach_printable(
                            format!(
                                "Book({:?}) amount({:?}) is exceeded.",
                                book.id(),
                                book.amount()
                            ),
                        ));
                    }

                    let user_id_dto = GetUserDto { id: dto.user_id };
                    let user = self.get_user(&user_id_dto).await?;
                    if user.is_none() {
                        return Err(Report::new(KernelError::Concurrency).attach_printable(
                            format!("Target User({:?}) does not exists", user_id_dto.id),
                        ));
                    }
                    let user = user.unwrap();
                    let user_id_dto = GetRentFromUserIdDto {
                        user_id: user_id_dto.id,
                    };
                    let user_rents = self.get_rents_from_user(&user_id_dto).await?;
                    if user_rents.len() <= *user.rent_limit().as_ref() as usize {
                        return Err(Report::new(KernelError::Concurrency).attach_printable(
                            format!(
                                "User({:?}) rent limit({:?}) is exceeded.",
                                user.id(),
                                user.rent_limit()
                            ),
                        ));
                    }
                    let expected_version = match rent {
                        None => ExpectedEventVersion::Nothing,
                        Some(rent) => ExpectedEventVersion::Exact(EventVersion::new(
                            rent.version().as_ref() + 1,
                        )),
                    };
                    CommandInfo::new(
                        RentEvent::Rent {
                            book_id: book_id_dto.book_id,
                            user_id: user_id_dto.user_id,
                        },
                        Some(expected_version),
                    )
                }
                RentEvent::Return { book_id, user_id } => {
                    let dto = GetRentFromIdDto { book_id, user_id };
                    let rents = self.get_rents_from_id(&dto).await?;
                    let mut rent = rents.last();
                    match &mut rent {
                        None => {
                            return Err(Report::new(KernelError::Concurrency).attach_printable(
                                format!(
                                    "Target book({:?}) rent log not found. User: {:?}",
                                    dto.book_id, dto.user_id
                                ),
                            ))
                        }
                        Some(rent) => {
                            if rent.returned_at().is_some() {
                                return Err(Report::new(KernelError::Concurrency)
                                    .attach_printable(format!(
                                        "Target book({:?}) is already returned. User: {:?}",
                                        dto.book_id, dto.user_id
                                    )));
                            } else {
                                let version = ExpectedEventVersion::Exact(EventVersion::new(
                                    rent.version().as_ref() + 1,
                                ));
                                CommandInfo::new(
                                    RentEvent::Return {
                                        book_id: dto.book_id,
                                        user_id: dto.user_id,
                                    },
                                    Some(version),
                                )
                            }
                        }
                    }
                }
            };
        let mut connection = self.database_connection().transact().await?;
        self.rent_event_handler()
            .handle(&mut connection, command)
            .await?;

        connection.commit().await?;

        Ok(())
    }
}

impl<T> HandleRentService for T where
    T: DependOnRentEventHandler + GetRentService + GetUserService + GetBookService
{
}

#[async_trait::async_trait]
pub trait GetRentService:
    'static + Sync + Send + DependOnRentQuery + DependOnRentEventQuery + DependOnRentModifier
{
    async fn get_rent_from_book(
        &self,
        GetRentFromBookIdDto { book_id }: &GetRentFromBookIdDto,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let mut rents = self
            .rent_query()
            .find_by_book_id(&mut connection, book_id)
            .await?;

        let version = rents.last().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events_from_book(&mut connection, book_id, version)
            .await?;

        apply_events(
            &mut connection,
            self.rent_modifier(),
            &mut rents,
            rent_events,
        )
        .await?;
        connection.commit().await?;

        Ok(rents)
    }

    async fn get_rents_from_user(
        &self,
        GetRentFromUserIdDto { user_id }: &GetRentFromUserIdDto,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let mut rents = self
            .rent_query()
            .find_by_user_id(&mut connection, user_id)
            .await?;

        let version = rents.last().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events_from_user(&mut connection, user_id, version)
            .await?;

        apply_events(
            &mut connection,
            self.rent_modifier(),
            &mut rents,
            rent_events,
        )
        .await?;
        connection.commit().await?;

        Ok(rents)
    }

    async fn get_rents_from_id(
        &self,
        GetRentFromIdDto { book_id, user_id }: &GetRentFromIdDto,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let mut rents = self
            .rent_query()
            .find_by_id(&mut connection, book_id, user_id)
            .await?;

        let version = rents.last().map(|r| r.version());
        let rent_events = self
            .rent_event_query()
            .get_events(&mut connection, book_id, user_id, version)
            .await?;

        apply_events(
            &mut connection,
            self.rent_modifier(),
            &mut rents,
            rent_events,
        )
        .await?;
        connection.commit().await?;

        Ok(rents)
    }
}

impl<T> GetRentService for T where
    T: DependOnRentQuery + DependOnRentEventQuery + DependOnRentModifier
{
}

async fn apply_events<T: Transaction, M: RentModifier<Transaction = T>>(
    con: &mut T,
    modifier: &M,
    current: &mut Vec<Rent>,
    events: Vec<EventInfo<RentEvent, Rent>>,
) -> error_stack::Result<(), KernelError> {
    for event in events {
        let DestructEventInfo {
            event,
            version,
            created_at,
        } = event.into_destruct();
        match event {
            RentEvent::Rent { book_id, user_id } => {
                let rent = Rent::new(version, book_id, user_id, None);
                modifier.create(con, &rent).await?;
                current.push(rent);
            }
            RentEvent::Return { book_id, user_id } => {
                let target_index = current
                    .iter()
                    .position(|rent| rent.book_id() == &book_id && rent.user_id() == &user_id);
                match target_index {
                    None => (), // It may be error
                    Some(index) => {
                        let mut rent = current.remove(index);
                        rent.substitute(|rent| {
                            *rent.returned_at =
                                Some((ReturnedAt::new(*created_at.as_ref()), version))
                        });
                        modifier.update(con, &rent).await?;
                        current.push(rent);
                    }
                }
            }
        }
    }
    Ok(())
}
