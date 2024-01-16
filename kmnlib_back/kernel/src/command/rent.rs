use crate::entity::{BookId, EventVersion, Rent, UserId};
use serde::{Deserialize, Serialize};

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
    type Error;
    async fn handle(&self, command: RentCommand) -> Result<EventVersion<Rent>, Self::Error>;
}

pub trait DependOnRentCommandHandler {
    type RentCommandHandler: RentCommandHandler;
    fn rent_command_handler(&self) -> &Self::RentCommandHandler;
}
