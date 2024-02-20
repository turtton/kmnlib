use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{BookId, EventVersion, Rent, UserId};
use crate::event::{EventInfo, RentEvent};
use crate::KernelError;

#[async_trait::async_trait]
pub trait RentQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn find_by_id(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Option<Rent>, KernelError>;
    async fn find_by_book_id(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
    ) -> error_stack::Result<Vec<Rent>, KernelError>;

    async fn find_by_user_id(
        &self,
        con: &mut Self::Transaction,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError>;
}

pub trait DependOnRentQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type RentQuery: RentQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn rent_query(&self) -> &Self::RentQuery;
}

#[async_trait::async_trait]
pub trait RentEventQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn get_events_from_book(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError>;

    async fn get_events_from_user(
        &self,
        con: &mut Self::Transaction,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError>;

    async fn get_events(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
        user_id: &UserId,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError>;
}

pub trait DependOnRentEventQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type RentEventQuery: RentEventQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn rent_event_query(&self) -> &Self::RentEventQuery;
}
