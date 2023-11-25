use crate::entity::{Book, BookId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookQuery: Sync + Send + 'static {
    async fn find_by_id(&self, id: &BookId) -> Result<Option<Book>, KernelError>;
}

pub trait DependOnBookQuery: Sync + Send + 'static {
    type BookQuery: BookQuery;
    fn book_query(&self) -> &Self::BookQuery;
}
