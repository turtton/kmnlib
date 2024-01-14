use crate::entity::{Book, BookId, EventVersion};
use crate::event::BookEvent;

#[async_trait::async_trait]
pub trait BookQuery<Connection>: Sync + Send + 'static {
    type Error;
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &BookId,
    ) -> Result<Option<Book>, Self::Error>;
}

pub trait DependOnBookQuery<Connection>: Sync + Send + 'static {
    type BookQuery: BookQuery<Connection>;
    fn book_query(&self) -> &Self::BookQuery;
}

#[async_trait::async_trait]
pub trait BookEventQuery: Sync + Send + 'static {
    type Error;
    async fn get_events(
        &self,
        id: &BookId,
        since: Option<EventVersion<Book>>,
    ) -> Result<Vec<BookEvent>, Self::Error>;
}

pub trait DependOnBookEventQuery: Sync + Send + 'static {
    type BookEventQuery: BookEventQuery;
    fn book_event_query(&self) -> &Self::BookEventQuery;
}
