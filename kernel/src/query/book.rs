use crate::entity::{Book, BookId, EventVersion};
use crate::event::BookEvent;
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait BookQuery<Connection>: Sync + Send + 'static {
    type Error: Context;
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &BookId,
    ) -> Result<Option<Book>, Report<Self::Error>>;
}

pub trait DependOnBookQuery<Connection>: Sync + Send + 'static {
    type BookQuery: BookQuery<Connection>;
    fn book_query(&self) -> &Self::BookQuery;
}

#[async_trait::async_trait]
pub trait BookEventQuery: Sync + Send + 'static {
    type Error: Context;
    async fn get_events(
        &self,
        id: &BookId,
        since: Option<EventVersion<Book>>,
    ) -> Result<Vec<BookEvent>, Report<Self::Error>>;
}

pub trait DependOnBookEventQuery: Sync + Send + 'static {
    type BookEventQuery: BookEventQuery;
    fn book_event_query(&self) -> &Self::BookEventQuery;
}
