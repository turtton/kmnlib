use crate::database::Transaction;
use crate::entity::{BookAmount, BookId, BookTitle};
use crate::KernelError;

#[derive(Debug, Clone, Eq, PartialEq)]
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
pub trait BookCommandHandler<Connection: Transaction> {
    async fn handle(
        &self,
        con: &mut Connection,
        command: BookCommand,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnBookCommandHandler<Connection: Transaction> {
    type BookCommandHandler: BookCommandHandler<Connection>;
    fn book_command_handler(&self) -> &Self::BookCommandHandler;
}
