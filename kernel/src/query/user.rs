use crate::entity::{User, UserId};
use crate::KernelError;
use error_stack::Report;

#[async_trait::async_trait]
pub trait UserQuery: Sync + Send + 'static {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, Report<KernelError>>;
}

pub trait DependOnUserQuery: Sync + Send + 'static {
    type UserQuery: UserQuery;
    fn user_query(&self) -> &Self::UserQuery;
}
