use destructure::Destructure;
use error_stack::Report;

use crate::entity::{Book, BookAmount, BookId, BookTitle};
use crate::event::{Applier, DestructEventInfo, EventInfo, EventRowFieldAttachments};
use crate::KernelError;

const BOOK_CREATED: &str = "book_created";
const BOOK_UPDATED: &str = "book_updated";
const BOOK_DELETED: &str = "book_deleted";

#[derive(Debug, Eq, PartialEq)]
pub enum BookEvent {
    Create {
        id: BookId,
        title: BookTitle,
        amount: BookAmount,
    },
    Update {
        id: BookId,
        title: Option<BookTitle>,
        amount: Option<BookAmount>,
    },
    Delete {
        id: BookId,
    },
}

impl Applier<EventInfo<BookEvent, Book>> for Option<Book> {
    fn apply(&mut self, event: EventInfo<BookEvent, Book>) {
        let DestructEventInfo { event, version, .. } = event.into_destruct();
        match (self, event) {
            (option @ None, BookEvent::Create { id, title, amount }) => {
                *option = Some(Book::new(BookId::new(id), title, amount, version));
            }
            (Some(book), BookEvent::Update { title, amount, .. }) => book.substitute(|book| {
                if let Some(title) = title {
                    *book.title = title;
                }
                if let Some(amount) = amount {
                    *book.amount = amount;
                }
                *book.version = version;
            }),
            (option, BookEvent::Delete { .. }) => {
                *option = None;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Destructure)]
pub struct BookEventRow {
    event_name: String,
    id: BookId,
    title: Option<BookTitle>,
    amount: Option<BookAmount>,
}

impl BookEventRow {
    pub fn new(
        event_name: String,
        id: BookId,
        title: Option<BookTitle>,
        amount: Option<BookAmount>,
    ) -> Self {
        Self {
            event_name,
            id,
            title,
            amount,
        }
    }
}

impl From<BookEvent> for BookEventRow {
    fn from(value: BookEvent) -> Self {
        match value {
            BookEvent::Create { id, title, amount } => {
                Self::new(String::from(BOOK_CREATED), id, Some(title), Some(amount))
            }
            BookEvent::Update { id, title, amount } => {
                Self::new(String::from(BOOK_UPDATED), id, title, amount)
            }
            BookEvent::Delete { id } => Self::new(String::from(BOOK_DELETED), id, None, None),
        }
    }
}

impl TryFrom<BookEventRow> for BookEvent {
    type Error = Report<KernelError>;
    fn try_from(value: BookEventRow) -> Result<Self, Self::Error> {
        let event_name = value.event_name;
        match &*event_name {
            BOOK_CREATED => {
                let id = value.id;
                let title = value.title.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "title")
                })?;
                let amount = value.amount.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "amount")
                })?;
                Ok(Self::Create { id, title, amount })
            }
            BOOK_UPDATED => Ok(Self::Update {
                id: value.id,
                title: value.title,
                amount: value.amount,
            }),
            BOOK_DELETED => Ok(Self::Delete { id: value.id }),
            _ => Err(Report::new(KernelError::Internal).attach_unknown_event("book", &event_name)),
        }
    }
}
