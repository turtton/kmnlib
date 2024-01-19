use crate::entity::{Book, BookId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookModifier<Connection>: 'static + Sync + Send {
    async fn create(
        &self,
        con: &mut Connection,
        book: Book,
    ) -> error_stack::Result<(), KernelError>;
    async fn update(
        &self,
        con: &mut Connection,
        book: Book,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Connection,
        book_id: BookId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnBookModifier<Connection>: 'static + Sync + Send {
    type BookModifier: BookModifier<Connection>;
    fn book_modifier(&self) -> &Self::BookModifier;
}
