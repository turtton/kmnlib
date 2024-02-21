use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{User, UserId};
use crate::event::{CommandInfo, UserEvent};
use crate::KernelError;

#[async_trait::async_trait]
pub trait UserEventHandler: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn handle(
        &self,
        con: &mut Self::Transaction,
        event: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<UserId, KernelError>;
}

pub trait DependOnUserEventHandler: Sync + Send + 'static + DependOnDatabaseConnection {
    type UserEventHandler: UserEventHandler<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn user_event_handler(&self) -> &Self::UserEventHandler;
}
