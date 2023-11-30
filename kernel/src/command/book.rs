use crate::entity::{Book, BookId, BookTitle, EventVersion};
use error_stack::{Context, Report};
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Display)]
pub enum BookCommand {
    CreateBook {
        title: BookTitle,
    },
    RentBook {
        id: BookId,
        rev_version: EventVersion<Book>,
    },
    ReturnBook {
        id: BookId,
        rev_version: EventVersion<Book>,
    },
    DeleteBook {
        id: BookId,
    },
}

#[async_trait::async_trait]
pub trait BookCommandHandler {
    type Error: Context;
    async fn handle(&self, command: BookCommand) -> Result<Uuid, Report<Self::Error>>;
}

pub trait DependOnBookCommandHandler {
    type BookCommandHandler: BookCommandHandler;
    fn book_command_handler(&self) -> &Self::BookCommandHandler;
}
