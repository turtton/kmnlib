use crate::entity::{EventVersion, User, UserId};
use crate::event::UserEvent;
use crate::KernelError;

#[async_trait::async_trait]
pub trait UserQuery<Connection>: Sync + Send + 'static {
    async fn find_by_id(
        &self,
        con: &mut Connection,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError>;
}

pub trait DependOnUserQuery<Connection>: Sync + Send + 'static {
    type UserQuery: UserQuery<Connection>;
    fn user_query(&self) -> &Self::UserQuery;
}

#[async_trait::async_trait]
pub trait UserEventQuery: Sync + Send + 'static {
    async fn get_events(
        &self,
        id: &UserId,
        since: Option<EventVersion<User>>,
    ) -> error_stack::Result<Vec<UserEvent>, KernelError>;
}

pub trait DependOnUserEventQuery: Sync + Send + 'static {
    type UserEventQuery: UserEventQuery;
    fn user_event_query(&self) -> &Self::UserEventQuery;
}
