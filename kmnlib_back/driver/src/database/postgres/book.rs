use error_stack::Report;
use sqlx::{types, PgConnection};
use uuid::Uuid;

use kernel::interface::command::{BookCommand, BookCommandHandler};
use kernel::interface::event::{BookEvent, EventInfo};
use kernel::interface::query::{BookEventQuery, BookQuery};
use kernel::interface::update::BookModifier;
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};
use kernel::KernelError;

use crate::database::postgres::PostgresConnection;
use crate::error::ConvertError;

pub struct PostgresBookRepository;

#[async_trait::async_trait]
impl BookQuery<PostgresConnection> for PostgresBookRepository {
    async fn find_by_id(
        &self,
        con: &mut PostgresConnection,
        id: &BookId,
    ) -> error_stack::Result<Option<Book>, KernelError> {
        PgBookInternal::find_by_id(con, id).await
    }
}

#[async_trait::async_trait]
impl BookModifier<PostgresConnection> for PostgresBookRepository {
    async fn create(
        &self,
        con: &mut PostgresConnection,
        book: Book,
    ) -> error_stack::Result<(), KernelError> {
        PgBookInternal::create(con, book).await
    }

    async fn update(
        &self,
        con: &mut PostgresConnection,
        book: Book,
    ) -> error_stack::Result<(), KernelError> {
        PgBookInternal::update(con, book).await
    }

    async fn delete(
        &self,
        con: &mut PostgresConnection,
        book_id: BookId,
    ) -> error_stack::Result<(), KernelError> {
        PgBookInternal::delete(con, book_id).await
    }
}

#[async_trait::async_trait]
impl BookCommandHandler<PostgresConnection> for PostgresBookRepository {
    async fn handle(
        &self,
        con: &mut PostgresConnection,
        command: BookCommand,
    ) -> error_stack::Result<(), KernelError> {
        PgBookInternal::handle_command(con, command).await
    }
}

#[async_trait::async_trait]
impl BookEventQuery<PostgresConnection> for PostgresBookRepository {
    async fn get_events(
        &self,
        con: &mut PostgresConnection,
        id: &BookId,
        since: Option<&EventVersion<Book>>,
    ) -> error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError> {
        PgBookInternal::get_events(con, id, since).await
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

#[derive(sqlx::FromRow)]
struct BookEventRow {
    id: i64,
    event: types::Json<BookEvent>,
}

impl From<BookEventRow> for EventInfo<BookEvent, Book> {
    fn from(value: BookEventRow) -> Self {
        EventInfo::new(value.event.0, EventVersion::new(value.id))
    }
}

pub(in crate::database) struct PgBookInternal;

impl PgBookInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        id: &BookId,
    ) -> error_stack::Result<Option<Book>, KernelError> {
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
        .await
        .map_err(|e| match e {
            sqlx::Error::PoolTimedOut => Report::from(e).change_context(KernelError::Timeout),
            _ => Report::from(e).change_context(KernelError::Internal),
        })?;
        let found = row.map(Book::from);
        Ok(found)
    }

    async fn create(con: &mut PgConnection, book: Book) -> error_stack::Result<(), KernelError> {
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
        .await
        .map_err(|e| match e {
            sqlx::Error::PoolTimedOut => Report::from(e).change_context(KernelError::Timeout),
            _ => Report::from(e).change_context(KernelError::Internal),
        })?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, book: Book) -> error_stack::Result<(), KernelError> {
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
        .convert_error()?;
        Ok(())
    }

    async fn delete(
        con: &mut PgConnection,
        book_id: BookId,
    ) -> error_stack::Result<(), KernelError> {
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
        .convert_error()?;
        Ok(())
    }

    async fn handle_command(
        con: &mut PgConnection,
        command: BookCommand,
    ) -> error_stack::Result<(), KernelError> {
        let (book_id, event_version, event) = BookEvent::convert(command);
        match event_version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO book_events (book_id, event)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(book_id.as_ref())
                .bind(types::Json::from(event))
                .execute(con)
                .await
                .convert_error()?;
            }
            Some(version) => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO book_events (id, book_id, event)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(version.as_ref())
                .bind(book_id.as_ref())
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
        id: &BookId,
        since: Option<&EventVersion<Book>>,
    ) -> error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError> {
        let row = match since {
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, BookEventRow>(
                    r#"
            SELECT id, event FROM book_events where id > $1 AND book_id = $2
            "#,
                )
                .bind(version.as_ref())
            }
            None => {
                // language=postgresql
                sqlx::query_as::<_, BookEventRow>(
                    r#"
            SELECT id, event FROM book_events where book_id = $1
            "#,
                )
            }
        }
        .bind(id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        Ok(row.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod test {
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::BookQuery;
    use kernel::interface::update::BookModifier;
    use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};
    use kernel::KernelError;

    use crate::database::postgres::book::PostgresBookRepository;
    use crate::database::postgres::PostgresDatabase;

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test() -> error_stack::Result<(), KernelError> {
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
