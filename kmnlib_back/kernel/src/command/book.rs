use serde::{Deserialize, Serialize};

use crate::entity::{Book, BookAmount, BookId, BookTitle, EventVersion};
use crate::KernelError;

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
    async fn handle(
        &self,
        command: BookCommand,
    ) -> error_stack::Result<EventVersion<Book>, KernelError>;
}

pub trait DependOnBookCommandHandler {
    type BookCommandHandler: BookCommandHandler;
    fn book_command_handler(&self) -> &Self::BookCommandHandler;
}
