use serde::{Deserialize, Serialize};

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
pub trait RentCommandHandler {
    async fn handle(
        &self,
        command: RentCommand,
    ) -> error_stack::Result<EventVersion<Rent>, KernelError>;
}

pub trait DependOnRentCommandHandler {
    type RentCommandHandler: RentCommandHandler;
    fn rent_command_handler(&self) -> &Self::RentCommandHandler;
}
