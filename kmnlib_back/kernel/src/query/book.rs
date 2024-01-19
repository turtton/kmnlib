use crate::entity::{Book, BookId, EventVersion};
use crate::event::BookEvent;
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookQuery<Connection>: Sync + Send + 'static {
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &BookId,
    ) -> error_stack::Result<Option<Book>, KernelError>;
}

pub trait DependOnBookQuery<Connection>: Sync + Send + 'static {
    type BookQuery: BookQuery<Connection>;
    fn book_query(&self) -> &Self::BookQuery;
}

#[async_trait::async_trait]
pub trait BookEventQuery: Sync + Send + 'static {
    async fn get_events(
        &self,
        id: &BookId,
        since: Option<EventVersion<Book>>,
    ) -> error_stack::Result<Vec<BookEvent>, KernelError>;
}

pub trait DependOnBookEventQuery: Sync + Send + 'static {
    type BookEventQuery: BookEventQuery;
    fn book_event_query(&self) -> &Self::BookEventQuery;
}
