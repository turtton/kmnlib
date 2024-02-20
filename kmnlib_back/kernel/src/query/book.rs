use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{Book, BookId, EventVersion};
use crate::event::{BookEvent, EventInfo};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn find_by_id(
        &self,
        con: &mut Self::Transaction,
        id: &BookId,
    ) -> error_stack::Result<Option<Book>, KernelError>;
}

pub trait DependOnBookQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type BookQuery: BookQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn book_query(&self) -> &Self::BookQuery;
}

#[async_trait::async_trait]
pub trait BookEventQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn get_events(
        &self,
        con: &mut Self::Transaction,
        id: &BookId,
        since: Option<&EventVersion<Book>>,
    ) -> error_stack::Result<Vec<EventInfo<BookEvent, Book>>, KernelError>;
}

pub trait DependOnBookEventQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type BookEventQuery: BookEventQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn book_event_query(&self) -> &Self::BookEventQuery;
}
