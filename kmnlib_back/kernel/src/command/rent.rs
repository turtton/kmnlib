use serde::{Deserialize, Serialize};

use crate::database::Transaction;
use crate::entity::{BookId, EventVersion, Rent, UserId};
use crate::KernelError;

pub static RENT_STREAM_NAME: &str = "rent-stream";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RentCommand {
    Rent {
        user_id: UserId,
        book_id: BookId,
        expected_version: EventVersion<Rent>,
    },
    Return {
        user_id: UserId,
        book_id: BookId,
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
