use serde::{Deserialize, Serialize};

use crate::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};

pub static BOOK_STREAM_NAME: &str = "book-stream";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BookCommand {
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

#[async_trait::async_trait]
pub trait BookCommandHandler {
    type Error;
    async fn handle(&self, command: BookCommand) -> Result<EventVersion<Book>, Self::Error>;
}

pub trait DependOnBookCommandHandler {
    type BookCommandHandler: BookCommandHandler;
    fn book_command_handler(&self) -> &Self::BookCommandHandler;
}
