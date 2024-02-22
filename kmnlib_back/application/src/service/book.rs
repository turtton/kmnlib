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

use crate::transfer::{GetAllBookDto, GetBookDto};

#[async_trait::async_trait]
pub trait HandleBookService: 'static + Sync + Send + DependOnBookEventHandler {
    async fn handle_event(&self, event: BookEvent) -> error_stack::Result<BookId, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let command = CommandInfo::new(event, None);
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
    async fn get_all(
        &self,
        GetAllBookDto { limit, offset }: GetAllBookDto,
    ) -> error_stack::Result<Vec<Book>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let mut books = self
            .book_query()
            .get_all(&mut connection, &limit, &offset)
            .await?;

        for book in &mut books {
            let events = self
                .book_event_query()
                .get_events(&mut connection, book.id(), Some(book.version()))
                .await?;
            if !events.is_empty() {
                events.into_iter().for_each(|e| book.apply(e));
                self.book_modifier().update(&mut connection, book).await?;
            }
        }

        connection.commit().await?;

        Ok(books)
    }

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
            (true, None) => self.book_modifier().delete(&mut connection, &id).await?, // Not reachable
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
