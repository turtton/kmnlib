use crate::command::RentCommand;
use crate::entity::{BookId, EventVersion, Rent, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RentEvent {
    Rented { book_id: BookId, user_id: UserId },
    Returned { book_id: BookId, user_id: UserId },
}

impl RentEvent {
    pub fn convert(command: RentCommand) -> (String, Option<EventVersion<Rent>>, Self) {
        match command {
            RentCommand::Rent {
                user_id,
                book_id,
                expected_version,
            } => {
                let event = Self::Rented { user_id, book_id };
                ("rented-book".to_string(), Some(expected_version), event)
            }
            RentCommand::Return {
                user_id,
                book_id,
                expected_version,
            } => {
                let event = Self::Returned { user_id, book_id };
                ("returned-book".to_string(), Some(expected_version), event)
            }
        }
    }
}
