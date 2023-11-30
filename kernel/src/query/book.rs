use crate::entity::{Book, BookId};
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait BookQuery<Connection>: Sync + Send + 'static {
    type Error: Context;
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &BookId,
    ) -> Result<Option<Book>, Report<Self::Error>>;
}

pub trait DependOnBookQuery<Connection>: Sync + Send + 'static {
    type BookQuery: BookQuery<Connection>;
    fn book_query(&self) -> &Self::BookQuery;
}
