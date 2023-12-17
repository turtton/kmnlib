use crate::error::DriverError;
use error_stack::{Report, ResultExt};
use kernel::interface::query::BookQuery;
use kernel::interface::update::BookModifier;
use kernel::prelude::entity::{Book, BookId, BookTitle, EventVersion};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use uuid::Uuid;

pub struct PostgresBookRepository;

#[async_trait::async_trait]
impl BookQuery<PoolConnection<Postgres>> for PostgresBookRepository {
    type Error = DriverError;
    async fn find_by_id(
        &self,
        con: &mut PoolConnection<Postgres>,
        id: &BookId,
    ) -> Result<Option<Book>, Report<Self::Error>> {
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
    ) -> Result<(), Report<Self::Error>> {
        PgBookInternal::create(con, book).await
    }

    async fn update(
        &self,
        con: &mut PoolConnection<Postgres>,
        book: Book,
    ) -> Result<(), Report<Self::Error>> {
        PgBookInternal::update(con, book).await
    }

    async fn delete(
        &self,
        con: &mut PoolConnection<Postgres>,
        book_id: BookId,
    ) -> Result<(), Report<Self::Error>> {
        PgBookInternal::delete(con, book_id).await
    }
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    version: i64,
}

impl From<BookRow> for Book {
    fn from(value: BookRow) -> Self {
        Book::new(
            BookId::new(value.id),
            BookTitle::new(value.title),
            EventVersion::new(value.version),
        )
    }
}

pub(in crate::database) struct PgBookInternal;

impl PgBookInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        id: &BookId,
    ) -> Result<Option<Book>, Report<DriverError>> {
        let row = sqlx::query_as::<_, BookRow>(
            // language=postgresql
            r#"
            SELECT id, title, version
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        let found = row.map(Book::from);
        Ok(found)
    }

    async fn create(con: &mut PgConnection, book: Book) -> Result<(), Report<DriverError>> {
        // language=postgresql
        sqlx::query(
            r#"
            INSERT INTO books (id, title, version)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(book.id().as_ref())
        .bind(book.title().as_ref())
        .bind(book.version().as_ref())
        .execute(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, book: Book) -> Result<(), Report<DriverError>> {
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
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }

    async fn delete(con: &mut PgConnection, book_id: BookId) -> Result<(), Report<DriverError>> {
        // language=postgresql
        sqlx::query(
            r#"
            DELETE FROM books
            WHERE id = $1
            "#,
        )
        .bind(book_id.as_ref())
        .execute(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::database::postgres::book::PostgresBookRepository;
    use crate::database::postgres::PostgresDatabase;
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::BookQuery;
    use kernel::interface::update::BookModifier;
    use kernel::prelude::entity::{Book, BookId, BookTitle, EventVersion};

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test() -> Result<(), Report<DriverError>> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let id = BookId::new(uuid::Uuid::new_v4());

        let book = Book::new(
            id.clone(),
            BookTitle::new("test".to_string()),
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
