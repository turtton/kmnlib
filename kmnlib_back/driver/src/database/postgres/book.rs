use error_stack::Report;
use sqlx::PgConnection;
use time::OffsetDateTime;
use uuid::Uuid;

use kernel::interface::command::{BookCommand, BookCommandHandler};
use kernel::interface::event::{BookEvent, BookEventRow, DestructBookEventRow, EventInfo};
use kernel::interface::query::{BookEventQuery, BookQuery};
use kernel::interface::update::BookModifier;
use kernel::KernelError;
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, CreatedAt, EventVersion};

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
struct BookEventRowColumn {
    version: i64,
    event_name: String,
    title: Option<String>,
    amount: Option<i32>,
    created_at: OffsetDateTime,
}

impl TryFrom<BookEventRowColumn> for EventInfo<BookEvent, Book> {
    type Error = Report<KernelError>;
    fn try_from(value: BookEventRowColumn) -> Result<Self, Self::Error> {
        let row = BookEventRow::new(
            value.event_name,
            value.title.map(BookTitle::new),
            value.amount.map(BookAmount::new),
        );
        let event = BookEvent::try_from(row)?;
        Ok(EventInfo::new(
            event,
            EventVersion::new(value.version),
            CreatedAt::new(value.created_at),
        ))
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
            SET title = $2, amount = $3, version = $4
            WHERE id = $1
            "#,
        )
        .bind(book.id().as_ref())
        .bind(book.title().as_ref())
        .bind(book.amount().as_ref())
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
        let DestructBookEventRow {
            event_name,
            title,
            amount,
        } = BookEventRow::from(event).into_destruct();
        let title_row = title.as_ref().map(AsRef::as_ref);
        let amount = amount.as_ref().map(AsRef::as_ref);
        match event_version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO book_events (book_id, event_name, title, amount)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(book_id.as_ref())
                .bind(event_name)
                .bind(title_row)
                .bind(amount)
                .execute(con)
                .await
                .convert_error()?;
            }
            Some(version) => {
                let mut version = version;
                if let EventVersion::Nothing = version {
                    let event = PgBookInternal::get_events(con, &book_id, None).await?;
                    if !event.is_empty() {
                        return Err(Report::new(KernelError::Concurrency)
                            .attach_printable("Event stream is already exists"));
                    } else {
                        version = EventVersion::new(1);
                    }
                }
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO book_events (version, book_id, event_name, title, amount)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(version.as_ref())
                .bind(book_id.as_ref())
                .bind(event_name)
                .bind(title_row)
                .bind(amount)
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
                sqlx::query_as::<_, BookEventRowColumn>(
                    r#"
            SELECT version, event_name, title, amount, created_at FROM book_events where version > $1 AND book_id = $2
            "#,
                )
                .bind(version.as_ref())
            }
            None => {
                // language=postgresql
                sqlx::query_as::<_, BookEventRowColumn>(
                    r#"
            SELECT version, event_name, title, amount, created_at FROM book_events where book_id = $1
            "#,
                )
            }
        }
        .bind(id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        row.into_iter().map(EventInfo::try_from).collect()
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use kernel::interface::command::{BookCommand, BookCommandHandler};
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::event::BookEvent;
    use kernel::interface::query::{BookEventQuery, BookQuery};
    use kernel::interface::update::BookModifier;
    use kernel::KernelError;
    use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};

    use crate::database::postgres::book::PostgresBookRepository;
    use crate::database::postgres::PostgresDatabase;

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test_query() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let id = BookId::new(Uuid::new_v4());

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

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test_event() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;

        let id = BookId::new(Uuid::new_v4());
        let title = BookTitle::new("test_book".to_string());
        let amount = BookAmount::new(0);

        let create_command = BookCommand::Create {
            id: id.clone(),
            title,
            amount,
        };
        PostgresBookRepository
            .handle(&mut con, create_command.clone())
            .await?;
        let create_event = PostgresBookRepository
            .get_events(&mut con, &id, None)
            .await?;
        let create_event = create_event.first().unwrap();
        let event_version_first = EventVersion::new(1);
        assert_eq!(create_event.version(), &event_version_first);
        let (_, _, expected_event) = BookEvent::convert(create_command);
        assert_eq!(create_event.event(), &expected_event);

        let update_command = BookCommand::Update {
            id: id.clone(),
            title: Some(BookTitle::new("test_book2".to_string())),
            amount: None,
        };
        PostgresBookRepository
            .handle(&mut con, update_command.clone())
            .await?;
        let update_event = PostgresBookRepository
            .get_events(&mut con, &id, Some(&event_version_first))
            .await?;
        let update_event = update_event.first().unwrap();
        assert_eq!(update_event.version(), &EventVersion::new(2));
        let (_, _, expected_event) = BookEvent::convert(update_command);
        assert_eq!(update_event.event(), &expected_event);

        // TODO: create book entity
        Ok(())
    }
}
