use destructure::Destructure;
use error_stack::Report;

use crate::command::RentCommand;
use crate::entity::{BookId, EventVersion, Rent, UserId};
use crate::event::{Applier, DestructEventInfo, EventInfo, EventRowFieldAttachments};
use crate::KernelError;

const BOOK_RENTED: &str = "book_rented";
const BOOK_RETURNED: &str = "book_returned";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RentEvent {
    Rented { book_id: BookId, user_id: UserId },
    Returned { book_id: BookId, user_id: UserId },
}

impl RentEvent {
    pub fn convert(command: RentCommand) -> (Option<EventVersion<Rent>>, Self) {
        match command {
            RentCommand::Rent {
                user_id,
                book_id,
                expected_version,
            } => {
                let event = Self::Rented { user_id, book_id };
                (Some(expected_version), event)
            }
            RentCommand::Return {
                user_id,
                book_id,
                expected_version,
            } => {
                let event = Self::Returned { user_id, book_id };
                (Some(expected_version), event)
            }
        }
    }
}

impl Applier<EventInfo<RentEvent, Rent>, ()> for Option<Rent> {
    fn apply(&mut self, event: EventInfo<RentEvent, Rent>, _id: ()) {
        let DestructEventInfo { event, version, .. } = event.into_destruct();
        match (self, event) {
            (option @ None, RentEvent::Rented { book_id, user_id }) => {
                *option = Some(Rent::new(version, book_id, user_id));
            }
            (option, RentEvent::Returned { .. }) => {
                *option = None;
            }
            _ => {}
        }
    }
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
            RentEvent::Rented { book_id, user_id } => {
                Self::new(String::from(BOOK_RENTED), book_id, user_id)
            }
            RentEvent::Returned { book_id, user_id } => {
                Self::new(String::from(BOOK_RETURNED), book_id, user_id)
            }
        }
    }
}

impl TryFrom<RentEventRow> for RentEvent {
    type Error = Report<KernelError>;
    fn try_from(row: RentEventRow) -> Result<Self, Self::Error> {
        match &*row.event_name {
            BOOK_RENTED => Ok(Self::Rented {
                book_id: row.book_id,
                user_id: row.user_id,
            }),
            BOOK_RETURNED => Ok(Self::Returned {
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
