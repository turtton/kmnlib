use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use uuid::Uuid;

use kernel::interface::query::BookQuery;
use kernel::interface::update::BookModifier;
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};

use crate::error::DriverError;

pub struct PostgresBookRepository;

#[async_trait::async_trait]
impl BookQuery<PoolConnection<Postgres>> for PostgresBookRepository {
    type Error = DriverError;
    async fn find_by_id(
        &self,
        con: &mut PoolConnection<Postgres>,
        id: &BookId,
    ) -> Result<Option<Book>, Self::Error> {
        PgBookInternal::find_by_id(con, id).await
    }
}

#[async_trait::async_trait]
impl BookModifier<PoolConnection<Postgres>> for PostgresBookRepository {
    type Error = DriverError;

    async fn create(
        &self,
        con: &mut PoolConnection<Postgres>,
        book: Book,
    ) -> Result<(), Self::Error> {
        PgBookInternal::create(con, book).await
    }

    async fn update(
        &self,
        con: &mut PoolConnection<Postgres>,
        book: Book,
    ) -> Result<(), Self::Error> {
        PgBookInternal::update(con, book).await
    }

    async fn delete(
        &self,
        con: &mut PoolConnection<Postgres>,
        book_id: BookId,
    ) -> Result<(), Self::Error> {
        PgBookInternal::delete(con, book_id).await
    }
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    amount: i32,
    version: i64,
}

impl From<BookRow> for Book {
    fn from(value: BookRow) -> Self {
        Book::new(
            BookId::new(value.id),
            BookTitle::new(value.title),
            BookAmount::new(value.amount),
            EventVersion::new(value.version),
        )
    }
}

pub(in crate::database) struct PgBookInternal;

impl PgBookInternal {
    async fn find_by_id(con: &mut PgConnection, id: &BookId) -> Result<Option<Book>, DriverError> {
        let row = sqlx::query_as::<_, BookRow>(
            // language=postgresql
            r#"
            SELECT id, title, amount, version
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(con)
        .await?;
        let found = row.map(Book::from);
        Ok(found)
    }

    async fn create(con: &mut PgConnection, book: Book) -> Result<(), DriverError> {
        // language=postgresql
        sqlx::query(
            r#"
            INSERT INTO books (id, title, amount, version)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(book.id().as_ref())
        .bind(book.title().as_ref())
        .bind(book.amount().as_ref())
        .bind(book.version().as_ref())
        .execute(con)
        .await?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, book: Book) -> Result<(), DriverError> {
        // language=postgresql
        sqlx::query(
            r#"
            UPDATE books
            SET title = $2, version = $3
            WHERE id = $1
            "#,
        )
        .bind(book.id().as_ref())
        .bind(book.title().as_ref())
        .bind(book.version().as_ref())
        .execute(con)
        .await?;
        Ok(())
    }

    async fn delete(con: &mut PgConnection, book_id: BookId) -> Result<(), DriverError> {
        // language=postgresql
        sqlx::query(
            r#"
            DELETE FROM books
            WHERE id = $1
            "#,
        )
        .bind(book_id.as_ref())
        .execute(con)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::BookQuery;
    use kernel::interface::update::BookModifier;
    use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};

    use crate::database::postgres::book::PostgresBookRepository;
    use crate::database::postgres::PostgresDatabase;
    use crate::error::DriverError;

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test() -> Result<(), DriverError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let id = BookId::new(uuid::Uuid::new_v4());

        let book = Book::new(
            id.clone(),
            BookTitle::new("test".to_string()),
            BookAmount::new(1),
            EventVersion::new(0),
        );
        PostgresBookRepository
            .create(&mut con, book.clone())
            .await?;

        let found = PostgresBookRepository.find_by_id(&mut con, &id).await?;
        assert_eq!(found, Some(book.clone()));

        let book = book.reconstruct(|b| b.title = BookTitle::new("test2".to_string()));
        PostgresBookRepository
            .update(&mut con, book.clone())
            .await?;

        let found = PostgresBookRepository.find_by_id(&mut con, &id).await?;
        assert_eq!(found, Some(book));

        PostgresBookRepository.delete(&mut con, id.clone()).await?;
        let found = PostgresBookRepository.find_by_id(&mut con, &id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
