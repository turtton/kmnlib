use crate::command::BookCommand;
use crate::entity::{Book, BookId, BookTitle, EventVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BookEvent {
    Created { title: BookTitle },
    Updated { title: Option<BookTitle> },
    Deleted,
}

impl BookEvent {
    pub fn convert(command: BookCommand) -> (String, BookId, Option<EventVersion<Book>>, Self) {
        match command {
            BookCommand::Create { id, title } => {
                let event = Self::Created { title };
                ("created-book".to_string(), id, None, event)
            }
            BookCommand::Update { id, title } => {
                let event = Self::Updated { title };
                ("updated-book".to_string(), id, None, event)
            }
            BookCommand::Delete { id } => {
                let event = Self::Deleted;
                ("deleted-book".to_string(), id, None, event)
            }
        }
    }
}
