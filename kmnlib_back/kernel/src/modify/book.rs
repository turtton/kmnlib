use crate::entity::{Book, BookId};

#[async_trait::async_trait]
pub trait BookModifier<Connection>: 'static + Sync + Send {
    type Error;
    async fn create(&self, con: &mut Connection, book: Book) -> Result<(), Self::Error>;
    async fn update(&self, con: &mut Connection, book: Book) -> Result<(), Self::Error>;
    async fn delete(&self, con: &mut Connection, book_id: BookId) -> Result<(), Self::Error>;
}

pub trait DependOnBookModifier<Connection>: 'static + Sync + Send {
    type BookModifier: BookModifier<Connection>;
    fn book_modifier(&self) -> &Self::BookModifier;
}
