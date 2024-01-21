use crate::database::Transaction;
use crate::entity::{Book, BookId, EventVersion};
use crate::event::{BookEvent, EventInfo};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookQuery<Connection: Transaction>: Sync + Send + 'static {
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &BookId,
    ) -> error_stack::Result<Option<Book>, KernelError>;
}

pub trait DependOnBookQuery<Connection: Transaction>: Sync + Send + 'static {
    type BookQuery: BookQuery<Connection>;
    fn book_query(&self) -> &Self::BookQuery;
}

#[async_trait::async_trait]
pub trait BookEventQuery<Connection: Transaction>: Sync + Send + 'static {
    async fn get_events(
        &self,
        con: &mut Connection,
        id: &BookId,
        since: Option<&EventVersion<Book>>,
    ) -> error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError>;
}

pub trait DependOnBookEventQuery<Connection: Transaction>: Sync + Send + 'static {
    type BookEventQuery: BookEventQuery<Connection>;
    fn book_event_query(&self) -> &Self::BookEventQuery;
}
