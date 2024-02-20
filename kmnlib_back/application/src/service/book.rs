use uuid::Uuid;

use kernel::interface::database::{DatabaseConnection, DependOnDatabaseConnection};
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
pub trait GetBookService:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection
    + DependOnBookQuery
    + DependOnBookModifier
    + DependOnBookEventQuery
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

impl<T> GetBookService for T where
    T: DependOnDatabaseConnection
        + DependOnBookQuery
        + DependOnBookModifier
        + DependOnBookEventQuery
{
}

#[async_trait::async_trait]
pub trait CreateBookService:
    'static + Sync + Send + DependOnDatabaseConnection + DependOnBookEventHandler
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

impl<T> CreateBookService for T where T: DependOnDatabaseConnection + DependOnBookEventHandler {}

#[async_trait::async_trait]
pub trait UpdateBookService:
    'static + Sync + Send + DependOnDatabaseConnection + DependOnBookEventHandler
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

impl<T> UpdateBookService for T where T: DependOnDatabaseConnection + DependOnBookEventHandler {}

#[async_trait::async_trait]
pub trait DeleteBookService:
    'static + Sync + Send + DependOnDatabaseConnection + DependOnBookEventHandler
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

impl<T> DeleteBookService for T where T: DependOnDatabaseConnection + DependOnBookEventHandler {}
