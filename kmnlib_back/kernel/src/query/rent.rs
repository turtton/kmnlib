use crate::database::Transaction;
use crate::entity::{BookId, EventVersion, Rent, UserId};
use crate::event::{EventInfo, RentEvent};
use crate::KernelError;

#[async_trait::async_trait]
pub trait RentQuery<Connection: Transaction>: Sync + Send + 'static {
    async fn find_by_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<Option<Rent>, KernelError>;
    async fn find_by_book_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
    ) -> error_stack::Result<Vec<Rent>, KernelError>;

    async fn find_by_user_id(
        &self,
        con: &mut Connection,
        user_id: &UserId,
    ) -> error_stack::Result<Vec<Rent>, KernelError>;
}

pub trait DependOnRentQuery<Connection: Transaction>: Sync + Send + 'static {
    type RentQuery: RentQuery<Connection>;
    fn rent_query(&self) -> &Self::RentQuery;
}

#[async_trait::async_trait]
pub trait RentEventQuery<Connection: Transaction>: Sync + Send + 'static {
    async fn get_events(
        &self,
        con: &mut Connection,
        since: Option<&EventVersion<Rent>>,
    ) -> error_stack::Result<Vec<EventInfo<RentEvent, Rent>>, KernelError>;
}

pub trait DependOnRentEventQuery<Connection: Transaction>: Sync + Send + 'static {
    type RentEventQuery: RentEventQuery<Connection>;
    fn rent_event_query(&self) -> &Self::RentEventQuery;
}
