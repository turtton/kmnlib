use uuid::Uuid;

use kernel::interface::database::{
    DependOnDatabaseConnection, QueryDatabaseConnection, Transaction,
};
use kernel::interface::event::{Applier, BookEvent, CommandInfo};
use kernel::interface::query::{
    BookEventQuery, BookQuery, DependOnBookEventQuery, DependOnBookQuery,
};
use kernel::interface::update::{
    BookEventHandler, BookModifier, DependOnBookEventHandler, DependOnBookModifier,
};
use kernel::prelude::entity::{BookAmount, BookId, BookTitle};
use kernel::KernelError;

use crate::transfer::{BookDto, DeleteBookDto, GetBookDto, UpdateBookDto};

#[async_trait::async_trait]
pub trait GetBookService<Connection: Transaction>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnBookQuery<Connection>
    + DependOnBookModifier<Connection>
    + DependOnBookEventQuery<Connection>
{
    async fn get_book(&self, dto: GetBookDto) -> error_stack::Result<Option<BookDto>, KernelError> {
        let mut connectioin = self.database_connection().transact().await?;

        let id = dto.id;
        let id = BookId::new(id);
        let mut book = self.book_query().find_by_id(&mut connectioin, &id).await?;
        let book_exists = book.is_some();

        let version = book.as_ref().map(|b| b.version());
        let book_events = self
            .book_event_query()
            .get_events(&mut connectioin, &id, version)
            .await?;

        book_events.into_iter().for_each(|event| book.apply(event));

        match (book_exists, &book) {
            (false, Some(book)) => self.book_modifier().create(&mut connectioin, book).await?,
            (true, Some(book)) => self.book_modifier().update(&mut connectioin, book).await?,
            (true, None) => self.book_modifier().delete(&mut connectioin, &id).await?,
            (false, None) => (),
        }

        match book {
            None => Ok(None),
            Some(book) => Ok(Some(BookDto::try_from(book)?)),
        }
    }
}

impl<Connection: Transaction, T> GetBookService<Connection> for T where
    T: DependOnDatabaseConnection<Connection>
        + DependOnBookQuery<Connection>
        + DependOnBookModifier<Connection>
        + DependOnBookEventQuery<Connection>
{
}

#[async_trait::async_trait]
pub trait CreateBookService<Connection: Transaction>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnBookEventHandler<Connection>
{
    async fn create_book(&self, dto: BookDto) -> error_stack::Result<Uuid, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let uuid = Uuid::new_v4();
        let id = BookId::new(uuid);
        let event = BookEvent::Create {
            id,
            title: BookTitle::new(dto.title),
            amount: BookAmount::new(dto.amount),
        };
        let command = CommandInfo::new(event, None);

        self.book_event_handler()
            .handle(&mut connection, command)
            .await?;

        Ok(uuid)
    }
}

impl<Connection: Transaction, T> CreateBookService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnBookEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait UpdateBookService<Connection: Transaction>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnBookEventHandler<Connection>
{
    async fn update_book(&self, dto: UpdateBookDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;
        let id = BookId::new(dto.id);
        let event = BookEvent::Update {
            id,
            title: dto.title.map(BookTitle::new),
            amount: dto.amount.map(BookAmount::new),
        };
        let command = CommandInfo::new(event, None);
        self.book_event_handler()
            .handle(&mut connection, command)
            .await?;
        Ok(())
    }
}

impl<Connection: Transaction, T> UpdateBookService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnBookEventHandler<Connection>
{
}

#[async_trait::async_trait]
pub trait DeleteBookService<Connection: Transaction>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnBookEventHandler<Connection>
{
    async fn delete_book(&self, dto: DeleteBookDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = BookId::new(dto.id);
        let event = BookEvent::Delete { id };
        let command = CommandInfo::new(event, None);

        self.book_event_handler()
            .handle(&mut connection, command)
            .await?;

        Ok(())
    }
}

impl<Connection: Transaction, T> DeleteBookService<Connection> for T where
    T: DependOnDatabaseConnection<Connection> + DependOnBookEventHandler<Connection>
{
}
