use serde::{Deserialize, Serialize};

use crate::command::BookCommand;
use crate::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
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
