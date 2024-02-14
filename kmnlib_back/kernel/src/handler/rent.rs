use crate::database::Transaction;
use crate::entity::{BookId, EventVersion, Rent, UserId};
use crate::event::{CommandInfo, RentEvent};
use crate::KernelError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RentCommand {
    Rent {
        book_id: BookId,
        user_id: UserId,
        expected_version: EventVersion<Rent>,
    },
    Return {
        book_id: BookId,
        user_id: UserId,
        expected_version: EventVersion<Rent>,
    },
}

#[async_trait::async_trait]
pub trait RentEventHandler<Connection: Transaction> {
    async fn handle(
        &self,
        con: &mut Connection,
        event: CommandInfo<RentEvent, Rent>,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnRentEventHandler<Connection: Transaction> {
    type RentEventHandler: RentEventHandler<Connection>;
    fn rent_event_handler(&self) -> &Self::RentEventHandler;
}
