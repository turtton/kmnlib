use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{Book, BookAmount, BookId, BookTitle};
use crate::event::{BookEvent, CommandInfo};
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
pub trait BookEventHandler: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn handle(
        &self,
        con: &mut Self::Transaction,
        event: CommandInfo<BookEvent, Book>,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnBookEventHandler: 'static + Sync + Send + DependOnDatabaseConnection {
    type BookEventHandler: BookEventHandler<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn book_event_handler(&self) -> &Self::BookEventHandler;
}
