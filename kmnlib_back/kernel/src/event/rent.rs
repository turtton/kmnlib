use destructure::Destructure;
use error_stack::Report;

use crate::entity::{BookId, UserId};
use crate::event::EventRowFieldAttachments;
use crate::KernelError;

const BOOK_RENTED: &str = "book_rented";
const BOOK_RETURNED: &str = "book_returned";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RentEvent {
    Rent { book_id: BookId, user_id: UserId },
    Return { book_id: BookId, user_id: UserId },
}

#[derive(Debug, Destructure)]
pub struct RentEventRow {
    event_name: String,
    book_id: BookId,
    user_id: UserId,
}

impl RentEventRow {
    pub fn new(event_name: String, book_id: BookId, user_id: UserId) -> Self {
        Self {
            event_name,
            book_id,
            user_id,
        }
    }
}

impl From<RentEvent> for RentEventRow {
    fn from(value: RentEvent) -> Self {
        match value {
            RentEvent::Rent { book_id, user_id } => {
                Self::new(String::from(BOOK_RENTED), book_id, user_id)
            }
            RentEvent::Return { book_id, user_id } => {
                Self::new(String::from(BOOK_RETURNED), book_id, user_id)
            }
        }
    }
}

impl TryFrom<RentEventRow> for RentEvent {
    type Error = Report<KernelError>;
    fn try_from(row: RentEventRow) -> Result<Self, Self::Error> {
        match &*row.event_name {
            BOOK_RENTED => Ok(Self::Rent {
                book_id: row.book_id,
                user_id: row.user_id,
            }),
            BOOK_RETURNED => Ok(Self::Return {
                book_id: row.book_id,
                user_id: row.user_id,
            }),
            _ => {
                Err(Report::new(KernelError::Internal)
                    .attach_unknown_event("rent", &row.event_name))
            }
        }
    }
}
