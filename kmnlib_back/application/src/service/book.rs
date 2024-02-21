use kernel::interface::database::{DatabaseConnection, Transaction};
use kernel::interface::event::{Applier, BookEvent, CommandInfo};
use kernel::interface::query::{
    BookEventQuery, BookQuery, DependOnBookEventQuery, DependOnBookQuery,
};
use kernel::interface::update::{
    BookEventHandler, BookModifier, DependOnBookEventHandler, DependOnBookModifier,
};
use kernel::prelude::entity::{Book, BookId};
use kernel::KernelError;

use crate::transfer::GetBookDto;

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
    async fn get_book(&self, dto: GetBookDto) -> error_stack::Result<Option<Book>, KernelError> {
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

        Ok(book)
    }
}

impl<T> GetBookService for T where
    T: DependOnBookQuery + DependOnBookModifier + DependOnBookEventQuery
{
}
