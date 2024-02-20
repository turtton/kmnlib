use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{EventVersion, User, UserId};
use crate::event::{EventInfo, UserEvent};
use crate::KernelError;

#[async_trait::async_trait]
pub trait UserQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn find_by_id(
        &self,
        con: &mut Self::Transaction,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError>;
}

pub trait DependOnUserQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type UserQuery: UserQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn user_query(&self) -> &Self::UserQuery;
}

#[async_trait::async_trait]
pub trait UserEventQuery: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn get_events(
        &self,
        con: &mut Self::Transaction,
        id: &UserId,
        since: Option<&EventVersion<User>>,
    ) -> error_stack::Result<Vec<EventInfo<UserEvent, User>>, KernelError>;
}

pub trait DependOnUserEventQuery: Sync + Send + 'static + DependOnDatabaseConnection {
    type UserEventQuery: UserEventQuery<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn user_event_query(&self) -> &Self::UserEventQuery;
}
