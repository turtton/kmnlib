use crate::database::Transaction;
use crate::entity::{BookId, EventVersion, Rent, UserId};
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
pub trait RentCommandHandler<Connection: Transaction> {
    async fn handle(
        &self,
        con: &mut Connection,
        command: RentCommand,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnRentCommandHandler<Connection: Transaction> {
    type RentCommandHandler: RentCommandHandler<Connection>;
    fn rent_command_handler(&self) -> &Self::RentCommandHandler;
}
