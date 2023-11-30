use crate::entity::{User, UserId};
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait UserQuery<Connection>: Sync + Send + 'static {
    type Error: Context;
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &UserId,
    ) -> Result<Option<User>, Report<Self::Error>>;
}

pub trait DependOnUserQuery<Connection>: Sync + Send + 'static {
    type UserQuery: UserQuery<Connection>;
    fn user_query(&self) -> &Self::UserQuery;
}
