use error_stack::Report;
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

use kernel::interface::event::{
    CommandInfo, DestructCommandInfo, DestructRentEventRow, EventInfo, RentEvent, RentEventRow,
};
use kernel::interface::query::{
    DependOnRentEventQuery, DependOnRentQuery, RentEventQuery, RentQuery,
};
use kernel::interface::update::{
    DependOnRentEventHandler, DependOnRentModifier, RentEventHandler, RentModifier,
};
use kernel::prelude::entity::{
    BookId, CreatedAt, EventVersion, ExpectedEventVersion, Rent, ReturnedAt, UserId,
};
use kernel::KernelError;

use crate::database::postgres::PostgresTransaction;
use crate::database::PostgresDatabase;
use crate::error::ConvertError;

pub struct PostgresRentRepository;

#[async_trait::async_trait]
impl RentQuery for PostgresRentRepository {
    type Transaction = PostgresTransaction;
    async fn find_by_id(
        &self,
        con: &mut PostgresTransaction,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        PgRentInternal::find_by_id(con, book_id, user_id).await
    }
    async fn find_by_book_id(
        &self,
        con: &mut PostgresTransaction,
        book_id: &BookId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        PgRentInternal::find_by_book_id(con, book_id).await
    }

    async fn find_by_user_id(
        &self,
        con: &mut PostgresTransaction,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        PgRentInternal::find_by_user_id(con, user_id).await
    }
}

impl DependOnRentQuery for PostgresDatabase {
    type RentQuery = PostgresRentRepository;
    fn rent_query(&self) -> &Self::RentQuery {
        &PostgresRentRepository
    }
}

#[async_trait::async_trait]
impl RentModifier for PostgresRentRepository {
    type Transaction = PostgresTransaction;
    async fn create(
        &self,
        con: &mut PostgresTransaction,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::create(con, rent).await
    }

    async fn update(
        &self,
        con: &mut Self::Transaction,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::update(con, rent).await
    }

    async fn delete(
        &self,
        con: &mut PostgresTransaction,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::delete(con, book_id, user_id).await
    }
}

impl DependOnRentModifier for PostgresDatabase {
    type RentModifier = PostgresRentRepository;
    fn rent_modifier(&self) -> &Self::RentModifier {
        &PostgresRentRepository
    }
}

#[async_trait::async_trait]
impl RentEventHandler for PostgresRentRepository {
    type Transaction = PostgresTransaction;
    async fn handle(
        &self,
        con: &mut PostgresTransaction,
        command: CommandInfo<RentEvent, Rent>,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::handle_command(con, command).await
    }
}

impl DependOnRentEventHandler for PostgresDatabase {
    type RentEventHandler = PostgresRentRepository;
    fn rent_event_handler(&self) -> &Self::RentEventHandler {
        &PostgresRentRepository
    }
}

#[async_trait::async_trait]
impl RentEventQuery for PostgresRentRepository {
    type Transaction = PostgresTransaction;
    async fn get_events_from_book(
        &self,
        con: &mut PostgresTransaction,
        book_id: &BookId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        PgRentInternal::get_events_from_book(con, book_id, since).await
    }

    async fn get_events_from_user(
        &self,
        con: &mut PostgresTransaction,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        PgRentInternal::get_events_from_user(con, user_id, since).await
    }

    async fn get_events(
        &self,
        con: &mut PostgresTransaction,
        book_id: &BookId,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        PgRentInternal::get_events(con, book_id, user_id, since).await
    }
}

impl DependOnRentEventQuery for PostgresDatabase {
    type RentEventQuery = PostgresRentRepository;
    fn rent_event_query(&self) -> &Self::RentEventQuery {
        &PostgresRentRepository
    }
}

#[derive(sqlx::FromRow)]
struct RentRow {
    version: i64,
    book_id: Uuid,
    user_id: Uuid,
    returned_at: Option<OffsetDateTime>,
    returned_version: Option<i64>,
}

impl TryFrom<RentRow> for Rent {
    type Error = Report<KernelError>;
    fn try_from(
        RentRow {
            version,
            book_id,
            user_id,
            returned_at,
            returned_version,
        }: RentRow,
    ) -> Result<Self, Self::Error> {
        let returned_at = match (returned_at, returned_version) {
            (Some(returned_at), Some(returned_version)) => Some((
                ReturnedAt::new(returned_at),
                EventVersion::new(returned_version),
            )),
            (None, None) => None,
            _ => {
                return Err(Report::new(KernelError::Internal).attach_printable(format!(
                "Invalid Rent data. version: {version}, book_id: {book_id:?}, user_id: {user_id:?}"
            )))
            }
        };
        Ok(Rent::new(
            EventVersion::new(version),
            BookId::new(book_id),
            UserId::new(user_id),
            returned_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct RentEventRowColumn {
    version: i64,
    event_name: String,
    book_id: Uuid,
    user_id: Uuid,
    created_at: OffsetDateTime,
}

impl TryFrom<RentEventRowColumn> for EventInfo<RentEvent, Rent> {
    type Error = Report<KernelError>;
    fn try_from(value: RentEventRowColumn) -> Result<Self, Self::Error> {
        let row = RentEventRow::new(
            value.event_name,
            BookId::new(value.book_id),
            UserId::new(value.user_id),
        );
        let event = RentEvent::try_from(row)?;
        Ok(EventInfo::new(
            event,
            EventVersion::new(value.version),
            CreatedAt::new(value.created_at),
        ))
    }
}

pub(in crate::database) struct PgRentInternal;

impl PgRentInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let row = sqlx::query_as::<_, RentRow>(
            // language=postgresql
            r#"
            SELECT
                version,
                book_id,
                user_id
            FROM
                book_rents
            WHERE
                book_id = $1 AND user_id = $2
            "#,
        )
        .bind(book_id.as_ref())
        .bind(user_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;
        row.into_iter()
            .map(Rent::try_from)
            .collect::<error_stack::Result<Vec<_>, KernelError>>()
    }

    async fn find_by_book_id(
        con: &mut PgConnection,
        book_id: &BookId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let row = sqlx::query_as::<_, RentRow>(
            // language=postgresql
            r#"
            SELECT
                book_id,
                user_id
            FROM
                book_rents
            WHERE
                book_id = $1
            "#,
        )
        .bind(book_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;
        row.into_iter()
            .map(Rent::try_from)
            .collect::<error_stack::Result<Vec<_>, KernelError>>()
    }

    async fn find_by_user_id(
        con: &mut PgConnection,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        let row = sqlx::query_as::<_, RentRow>(
            // language=postgresql
            r#"
            SELECT
                book_id,
                user_id
            FROM
                book_rents
            WHERE
                user_id = $1
            "#,
        )
        .bind(user_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;
        row.into_iter()
            .map(Rent::try_from)
            .collect::<error_stack::Result<_, KernelError>>()
    }

    async fn create(con: &mut PgConnection, rent: &Rent) -> error_stack::Result<(), KernelError> {
        sqlx::query(
            // language=postgresql
            r#"
            INSERT INTO book_rents (book_id, user_id, version)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(rent.book_id().as_ref())
        .bind(rent.user_id().as_ref())
        .bind(rent.version().as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, rent: &Rent) -> error_stack::Result<(), KernelError> {
        let (returned_at, returned_version) = match rent.returned_at() {
            None => (None, None),
            Some((returned_at, returned_version)) => (Some(returned_at), Some(returned_version)),
        };
        sqlx::query(
            // language=postgresql
            r#"
            UPDATE book_rents
            SET returned_at = $4, returned_version = 5
            WHERE version = $1 AND book_id = $2 AND user_id = $3
            "#,
        )
        .bind(rent.version().as_ref())
        .bind(rent.book_id().as_ref())
        .bind(rent.user_id().as_ref())
        .bind(returned_at.map(ReturnedAt::as_ref))
        .bind(returned_version.map(EventVersion::as_ref))
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn delete(
        con: &mut PgConnection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError> {
        sqlx::query(
            // language=postgresql
            r#"
            DELETE FROM book_rents
            WHERE book_id = $1 AND user_id = $2
            "#,
        )
        .bind(book_id.as_ref())
        .bind(user_id.as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn handle_command(
        con: &mut PgConnection,
        command: CommandInfo<RentEvent, Rent>,
    ) -> error_stack::Result<(), KernelError> {
        let DestructCommandInfo { event, version } = command.into_destruct();
        let DestructRentEventRow {
            user_id,
            book_id,
            event_name,
        } = RentEventRow::from(event).into_destruct();
        match version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO rent_events (book_id, user_id, event_name)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(book_id.as_ref())
                .bind(user_id.as_ref())
                .bind(event_name)
                .execute(con)
                .await
                .convert_error()?;
            }
            Some(version) => {
                let version = match version {
                    ExpectedEventVersion::Nothing => {
                        let event =
                            PgRentInternal::get_events(con, &book_id, &user_id, None).await?;
                        if !event.is_empty() {
                            return Err(Report::new(KernelError::Concurrency)
                                .attach_printable("Event stream already exists"));
                        } else {
                            EventVersion::new(1)
                        }
                    }
                    ExpectedEventVersion::Exact(version) => version,
                };

                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO rent_events (version, book_id, user_id, event_name)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(version.as_ref())
                .bind(book_id.as_ref())
                .bind(user_id.as_ref())
                .bind(event_name)
                .execute(con)
                .await
                .convert_error()?;
            }
        }
        Ok(())
    }
    async fn get_events_from_book(
        con: &mut PgConnection,
        book_id: &BookId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        let row = match since {
            None => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE book_id = $1
                    "#,
                )
            }
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE version > $1 AND book_id = $2
                    "#,
                )
                .bind(version.as_ref())
            }
        }
        .bind(book_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        row.into_iter().map(EventInfo::try_from).collect()
    }

    async fn get_events_from_user(
        con: &mut PgConnection,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        let row = match since {
            None => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE user_id = $1
                    "#,
                )
            }
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE version > $1 AND user_id = $2
                    "#,
                )
                .bind(version.as_ref())
            }
        }
        .bind(user_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        row.into_iter().map(EventInfo::try_from).collect()
    }

    async fn get_events(
        con: &mut PgConnection,
        book_id: &BookId,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        let row = match since {
            None => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE user_id = $1 AND book_id = $2
                    "#,
                )
            }
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRowColumn>(
                    r#"
                    SELECT version, event_name, book_id, user_id, created_at
                    FROM rent_events
                    WHERE version > $1 AND user_id = $2 AND book_id = $3
                    "#,
                )
                .bind(version.as_ref())
            }
        }
        .bind(user_id.as_ref())
        .bind(book_id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        row.into_iter().map(EventInfo::try_from).collect()
    }
}

#[cfg(test)]
mod test {
    use kernel::interface::database::DatabaseConnection;
    use kernel::interface::event::{CommandInfo, RentEvent};
    use kernel::interface::query::{RentEventQuery, RentQuery};
    use kernel::interface::update::{BookModifier, RentEventHandler, RentModifier, UserModifier};
    use kernel::prelude::entity::{
        Book, BookAmount, BookId, BookTitle, EventVersion, ExpectedEventVersion, IsDeleted, Rent,
        User, UserId, UserName, UserRentLimit,
    };
    use kernel::KernelError;

    use crate::database::postgres::{
        PostgresBookRepository, PostgresDatabase, PostgresRentRepository, PostgresUserRepository,
    };

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test_query() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let book_id = BookId::new(uuid::Uuid::new_v4());
        let book = Book::new(
            book_id.clone(),
            BookTitle::new("title".to_string()),
            BookAmount::new(1),
            EventVersion::new(0),
            IsDeleted::new(false),
        );
        PostgresBookRepository.create(&mut con, &book).await?;

        let user_id = UserId::new(uuid::Uuid::new_v4());
        let user = User::new(
            user_id.clone(),
            UserName::new("name".to_string()),
            UserRentLimit::new(1),
            EventVersion::new(0),
            IsDeleted::new(false),
        );
        PostgresUserRepository.create(&mut con, &user).await?;

        let rent = Rent::new(EventVersion::new(1), book_id.clone(), user_id.clone(), None);
        PostgresRentRepository.create(&mut con, &rent).await?;

        let find = PostgresRentRepository
            .find_by_id(&mut con, &book_id, &user_id)
            .await?;
        assert_eq!(find.get(0), Some(&rent));

        PostgresRentRepository
            .delete(&mut con, &book_id, &user_id)
            .await?;

        let find = PostgresRentRepository
            .find_by_id(&mut con, &book_id, &user_id)
            .await?;
        assert!(find.is_empty());
        Ok(())
    }

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test_event() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;

        let book_id = BookId::new(uuid::Uuid::new_v4());
        let user_id = UserId::new(uuid::Uuid::new_v4());

        let rent_event = RentEvent::Rent {
            book_id: book_id.clone(),
            user_id: user_id.clone(),
        };
        let rent_command = CommandInfo::new(rent_event, Some(ExpectedEventVersion::Nothing));
        PostgresRentRepository
            .handle(&mut con, rent_command.clone())
            .await?;
        let rent_event = PostgresRentRepository
            .get_events(&mut con, &book_id, &user_id, None)
            .await?;
        let rent_event = rent_event.first().unwrap();
        let event_version_first = EventVersion::new(1);
        assert_eq!(rent_event.version(), &event_version_first);
        assert_eq!(rent_event.event(), &rent_command.into_destruct().event);

        let return_event = RentEvent::Return {
            book_id: book_id.clone(),
            user_id: user_id.clone(),
        };
        let return_command = CommandInfo::new(
            return_event,
            Some(ExpectedEventVersion::Exact(EventVersion::new(2))),
        );
        PostgresRentRepository
            .handle(&mut con, return_command.clone())
            .await?;
        let rent_event = PostgresRentRepository
            .get_events(&mut con, &book_id, &user_id, Some(&event_version_first))
            .await?;
        let rent_event = rent_event.first().unwrap();
        assert_eq!(rent_event.version(), &EventVersion::new(2));
        assert_eq!(rent_event.event(), &return_command.into_destruct().event);

        // TODO: create rent entity
        Ok(())
    }
}
