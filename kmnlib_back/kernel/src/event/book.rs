use destructure::Destructure;
use error_stack::Report;

use crate::command::BookCommand;
use crate::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};
use crate::event::{Applier, DestructEventInfo, EventInfo, EventRowFieldAttachments};
use crate::KernelError;

const BOOK_CREATED: &str = "book_created";
const BOOK_UPDATED: &str = "book_updated";
const BOOK_DELETED: &str = "book_deleted";

#[derive(Debug, Eq, PartialEq)]
pub enum BookEvent {
    Created {
        title: BookTitle,
        amount: BookAmount,
    },
    Updated {
        title: Option<BookTitle>,
        amount: Option<BookAmount>,
    },
    Deleted,
}

impl BookEvent {
    pub fn convert(command: BookCommand) -> (BookId, Option<EventVersion<Book>>, Self) {
        match command {
            BookCommand::Create { id, title, amount } => {
                let event = Self::Created { title, amount };
                (id, None, event)
            }
            BookCommand::Update { id, title, amount } => {
                let event = Self::Updated { title, amount };
                (id, None, event)
            }
            BookCommand::Delete { id } => {
                let event = Self::Deleted;
                (id, None, event)
            }
        }
    }
}

impl Applier<EventInfo<BookEvent, Book>, BookId> for Option<Book> {
    fn apply(&mut self, event_info: EventInfo<BookEvent, Book>, id: BookId) {
        let DestructEventInfo { event, version, .. } = event_info.into_destruct();
        match (self, event) {
            (option @ None, BookEvent::Created { title, amount }) => {
                *option = Some(Book::new(BookId::new(id), title, amount, version));
            }
            (Some(book), BookEvent::Updated { title, amount }) => book.substitute(|book| {
                if let Some(title) = title {
                    *book.title = title;
                }
                if let Some(amount) = amount {
                    *book.amount = amount;
                }
                *book.version = version;
            }),
            (option, BookEvent::Deleted) => {
                *option = None;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Destructure)]
pub struct BookEventRow {
    event_name: String,
    title: Option<BookTitle>,
    amount: Option<BookAmount>,
}

impl BookEventRow {
    pub fn new(event_name: String, title: Option<BookTitle>, amount: Option<BookAmount>) -> Self {
        Self {
            event_name,
            title,
            amount,
        }
    }
}

impl From<BookEvent> for BookEventRow {
    fn from(value: BookEvent) -> Self {
        match value {
            BookEvent::Created { title, amount } => {
                Self::new(String::from(BOOK_CREATED), Some(title), Some(amount))
            }
            BookEvent::Updated { title, amount } => {
                Self::new(String::from(BOOK_UPDATED), title, amount)
            }
            BookEvent::Deleted => Self::new(String::from(BOOK_DELETED), None, None),
        }
    }
}

impl TryFrom<BookEventRow> for BookEvent {
    type Error = Report<KernelError>;
    fn try_from(value: BookEventRow) -> Result<Self, Self::Error> {
        let event_name = value.event_name;
        match &*event_name {
            BOOK_CREATED => {
                let title = value.title.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "title")
                })?;
                let amount = value.amount.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "amount")
                })?;
                Ok(Self::Created { title, amount })
            }
            BOOK_UPDATED => Ok(Self::Updated {
                title: value.title,
                amount: value.amount,
            }),
            BOOK_DELETED => Ok(Self::Deleted),
            _ => Err(Report::new(KernelError::Internal).attach_unknown_event("book", &event_name)),
        }
    }
}
