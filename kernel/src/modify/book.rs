use crate::entity::{Book, BookId};
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait BookModifier<Connection>: 'static + Sync + Send {
    type Error: Context;
    async fn create(&self, con: &mut Connection, book: Book) -> Result<(), Report<Self::Error>>;
    async fn update(&self, con: &mut Connection, book: Book) -> Result<(), Report<Self::Error>>;
    async fn delete(
        &self,
        con: &mut Connection,
        book_id: BookId,
    ) -> Result<(), Report<Self::Error>>;
}

pub trait DependOnBookModifier<Connection>: 'static + Sync + Send {
    type BookModifier: BookModifier<Connection>;
    fn book_modifier(&self) -> &Self::BookModifier;
}
