use crate::command::BookCommand;
use crate::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub fn convert(command: BookCommand) -> (String, BookId, Option<EventVersion<Book>>, Self) {
        match command {
            BookCommand::Create { id, title, amount } => {
                let event = Self::Created { title, amount };
                ("created-book".to_string(), id, None, event)
            }
            BookCommand::Update { id, title, amount } => {
                let event = Self::Updated { title, amount };
                ("updated-book".to_string(), id, None, event)
            }
            BookCommand::Delete { id } => {
                let event = Self::Deleted;
                ("deleted-book".to_string(), id, None, event)
            }
        }
    }
}
