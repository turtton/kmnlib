use sqlx::PgConnection;
use uuid::Uuid;

use kernel::interface::query::RentQuery;
use kernel::interface::update::RentModifier;
use kernel::prelude::entity::{BookId, Rent, UserId};
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
