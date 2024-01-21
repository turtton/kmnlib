use kernel::interface::command::{RentCommand, RentCommandHandler};
use kernel::interface::event::{EventInfo, RentEvent};
use sqlx::{types, PgConnection};
use uuid::Uuid;

use kernel::interface::query::{RentEventQuery, RentQuery};
use kernel::interface::update::RentModifier;
use kernel::prelude::entity::{BookId, EventVersion, Rent, UserId};
use kernel::KernelError;

use crate::database::postgres::PostgresConnection;
use crate::error::ConvertError;

pub struct PostgresRentRepository;

#[async_trait::async_trait]
impl RentQuery<PostgresConnection> for PostgresRentRepository {
    async fn find_by_id(
        &self,
        con: &mut PostgresConnection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Option<Rent>, KernelError> {
        PgRentInternal::find_by_id(con, book_id, user_id).await
    }
    async fn find_by_book_id(
        &self,
        con: &mut PostgresConnection,
        book_id: &BookId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        PgRentInternal::find_by_book_id(con, book_id).await
    }

    async fn find_by_user_id(
        &self,
        con: &mut PostgresConnection,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError> {
        PgRentInternal::find_by_user_id(con, user_id).await
    }
}

#[async_trait::async_trait]
impl RentModifier<PostgresConnection> for PostgresRentRepository {
    async fn create(
        &self,
        con: &mut PostgresConnection,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::create(con, rent).await
    }

    async fn delete(
        &self,
        con: &mut PostgresConnection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::delete(con, book_id, user_id).await
    }
}

#[async_trait::async_trait]
impl RentCommandHandler<PostgresConnection> for PostgresRentRepository {
    async fn handle(
        &self,
        con: &mut PostgresConnection,
        command: RentCommand,
    ) -> error_stack::Result<(), KernelError> {
        PgRentInternal::handle_command(con, command).await
    }
}

#[async_trait::async_trait]
impl RentEventQuery<PostgresConnection> for PostgresRentRepository {
    async fn get_events(
        &self,
        con: &mut PostgresConnection,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        PgRentInternal::get_events(con, since).await
    }
}

#[derive(sqlx::FromRow)]
struct RentRow {
    book_id: Uuid,
    user_id: Uuid,
}

impl From<RentRow> for Rent {
    fn from(value: RentRow) -> Self {
        Rent::new(BookId::new(value.book_id), UserId::new(value.user_id))
    }
}

#[derive(sqlx::FromRow)]
struct RentEventRow {
    id: i64,
    event: types::Json<RentEvent>,
}

impl From<RentEventRow> for EventInfo<RentEvent, Rent> {
    fn from(value: RentEventRow) -> Self {
        EventInfo::new(value.event.0, EventVersion::new(value.id))
    }
}

pub(in crate::database) struct PgRentInternal;

impl PgRentInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Option<Rent>, KernelError> {
        let row = sqlx::query_as::<_, RentRow>(
            // language=postgresql
            r#"
            SELECT
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
        .fetch_optional(con)
        .await
        .convert_error()?;
        Ok(row.map(Rent::from))
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
        Ok(row.into_iter().map(Rent::from).collect())
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
        Ok(row.into_iter().map(Rent::from).collect())
    }

    async fn create(con: &mut PgConnection, rent: &Rent) -> error_stack::Result<(), KernelError> {
        sqlx::query(
            // language=postgresql
            r#"
            INSERT INTO book_rents (book_id, user_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(rent.book_id().as_ref())
        .bind(rent.user_id().as_ref())
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
        command: RentCommand,
    ) -> error_stack::Result<(), KernelError> {
        let (event_version, event) = RentEvent::convert(command);
        match event_version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO rent_events (event)
                    VALUES ($1)
                    "#,
                )
                .bind(types::Json::from(event))
                .execute(con)
                .await
                .convert_error()?;
            }
            Some(version) => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO rent_events (id, event)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(version.as_ref())
                .bind(types::Json::from(event))
                .execute(con)
                .await
                .convert_error()?;
            }
        }
        Ok(())
    }

    async fn get_events(
        con: &mut PgConnection,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError> {
        let row = match since {
            None => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRow>(
                    r#"
                    SELECT id, event
                    FROM rent_events
                    "#,
                )
            }
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, RentEventRow>(
                    r#"
                    SELECT id, event
                    FROM rent_events
                    WHERE id > $1
                    "#,
                )
                .bind(version.as_ref())
            }
        }
        .fetch_all(con)
        .await
        .convert_error()?;

        Ok(row.into_iter().map(EventInfo::from).collect())
    }
}

#[cfg(test)]
mod test {
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::RentQuery;
    use kernel::interface::update::{BookModifier, RentModifier, UserModifier};
    use kernel::prelude::entity::{
        Book, BookAmount, BookId, BookTitle, EventVersion, Rent, User, UserId, UserName,
        UserRentLimit,
    };
    use kernel::KernelError;

    use crate::database::postgres::{
        PostgresBookRepository, PostgresDatabase, PostgresRentRepository, PostgresUserRepository,
    };

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let book_id = BookId::new(uuid::Uuid::new_v4());
        let book = Book::new(
            book_id.clone(),
            BookTitle::new("title".to_string()),
            BookAmount::new(1),
            EventVersion::new(0),
        );
        PostgresBookRepository.create(&mut con, book).await?;

        let user_id = UserId::new(uuid::Uuid::new_v4());
        let user = User::new(
            user_id.clone(),
            UserName::new("name".to_string()),
            UserRentLimit::new(1),
            EventVersion::new(0),
        );
        PostgresUserRepository.create(&mut con, user).await?;

        let rent = Rent::new(book_id.clone(), user_id.clone());
        PostgresRentRepository.create(&mut con, &rent).await?;

        let find = PostgresRentRepository
            .find_by_id(&mut con, &book_id, &user_id)
            .await?;
        assert_eq!(find, Some(rent.clone()));

        PostgresRentRepository
            .delete(&mut con, &book_id, &user_id)
            .await?;

        let find = PostgresRentRepository
            .find_by_id(&mut con, &book_id, &user_id)
            .await?;
        assert!(find.is_none());
        Ok(())
    }
}
