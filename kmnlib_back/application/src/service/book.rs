use uuid::Uuid;

use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{Applier, BookEvent, CommandInfo};
use kernel::interface::query::{
    BookEventQuery, BookQuery, DependOnBookEventQuery, DependOnBookQuery,
};
use kernel::interface::update::{
    BookEventHandler, BookModifier, DependOnBookEventHandler, DependOnBookModifier,
};
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle};
use kernel::KernelError;

use crate::transfer::{BookDto, DeleteBookDto, GetBookDto, UpdateBookDto};

#[async_trait::async_trait]
pub trait HandleBookService: 'static + Sync + Send + DependOnBookEventHandler {
    async fn handle_command(
        &self,
        command: CommandInfo<BookEvent, Book>,
    ) -> error_stack::Result<BookId, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = self
            .book_event_handler()
            .handle(&mut connection, command)
            .await?;

        connection.commit().await?;

        Ok(id)
    }
}

impl<T> HandleBookService for T where T: DependOnBookEventHandler {}

#[async_trait::async_trait]
pub trait GetBookService:
    'static + Sync + Send + DependOnBookQuery + DependOnBookModifier + DependOnBookEventQuery
{
    async fn get_book(&self, dto: GetBookDto) -> error_stack::Result<Option<BookDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = dto.id;
        let id = BookId::new(id);
        let mut book = self.book_query().find_by_id(&mut connection, &id).await?;
        let book_exists = book.is_some();

        let version = book.as_ref().map(|b| b.version());
        let book_events = self
            .book_event_query()
            .get_events(&mut connection, &id, version)
            .await?;

        book_events.into_iter().for_each(|event| book.apply(event));

        match (book_exists, &book) {
            (false, Some(book)) => self.book_modifier().create(&mut connection, book).await?,
            (true, Some(book)) => self.book_modifier().update(&mut connection, book).await?,
            (true, None) => self.book_modifier().delete(&mut connection, &id).await?,
            (false, None) => (),
        }
        connection.commit().await?;

        match book {
            None => Ok(None),
            Some(book) => Ok(Some(BookDto::try_from(book)?)),
        }
    }
}

impl<T> GetBookService for T where
    T: DependOnBookQuery + DependOnBookModifier + DependOnBookEventQuery
{
}

#[async_trait::async_trait]
pub trait CreateBookService: 'static + Sync + Send + DependOnBookEventHandler {
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
        connection.commit().await?;

        Ok(uuid)
    }
}

impl<T> CreateBookService for T where T: DependOnBookEventHandler {}

#[async_trait::async_trait]
pub trait UpdateBookService: 'static + Sync + Send + DependOnBookEventHandler {
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
        connection.commit().await?;

        Ok(())
    }
}

impl<T> UpdateBookService for T where T: DependOnBookEventHandler {}

#[async_trait::async_trait]
pub trait DeleteBookService: 'static + Sync + Send + DependOnBookEventHandler {
    async fn delete_book(&self, dto: DeleteBookDto) -> error_stack::Result<(), KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = BookId::new(dto.id);
        let event = BookEvent::Delete { id };
        let command = CommandInfo::new(event, None);

        self.book_event_handler()
            .handle(&mut connection, command)
            .await?;
        connection.commit().await?;

        Ok(())
    }
}

impl<T> DeleteBookService for T where T: DependOnBookEventHandler {}
