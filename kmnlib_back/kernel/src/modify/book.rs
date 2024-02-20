use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{Book, BookId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookModifier: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn create(
        &self,
        con: &mut Self::Transaction,
        book: &Book,
    ) -> error_stack::Result<(), KernelError>;
    async fn update(
        &self,
        con: &mut Self::Transaction,
        book: &Book,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnBookModifier: 'static + Sync + Send + DependOnDatabaseConnection {
    type BookModifier: BookModifier<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn book_modifier(&self) -> &Self::BookModifier;
}
