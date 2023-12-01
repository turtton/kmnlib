use crate::entity::{Book, BookId, BookTitle, EventVersion};
use error_stack::{Context, Report};
use serde::{Deserialize, Serialize};

pub static BOOK_STREAM_NAME: &str = "book-stream";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BookCommand {
    Create {
        id: BookId,
        title: BookTitle,
    },
    Rent {
        id: BookId,
        rev_version: EventVersion<Book>,
    },
    Return {
        id: BookId,
        rev_version: EventVersion<Book>,
    },
    Delete {
        id: BookId,
    },
}

#[async_trait::async_trait]
pub trait BookCommandHandler {
    type Error: Context;
    async fn handle(&self, command: BookCommand)
        -> Result<EventVersion<Book>, Report<Self::Error>>;
}

pub trait DependOnBookCommandHandler {
    type BookCommandHandler: BookCommandHandler;
    fn book_command_handler(&self) -> &Self::BookCommandHandler;
}
