use crate::entity::{Book, BookId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait BookQuery: Sync + Send + 'static {
    async fn find_by_id(&self, id: &BookId) -> Result<Option<Book>, KernelError>;
}
