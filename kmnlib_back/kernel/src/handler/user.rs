use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{User, UserId, UserName, UserRentLimit};
use crate::event::{CommandInfo, UserEvent};
use crate::KernelError;

#[derive(Debug, Clone)]
pub enum UserCommand {
    Create {
        id: UserId,
        name: UserName,
        rent_limit: UserRentLimit,
    },
    Update {
        id: UserId,
        name: Option<UserName>,
        rent_limit: Option<UserRentLimit>,
    },
    Delete {
        id: UserId,
    },
}

#[async_trait::async_trait]
pub trait UserEventHandler: Sync + Send + 'static {
    type Transaction: Transaction;
    async fn handle(
        &self,
        con: &mut Self::Transaction,
        event: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnUserEventHandler: Sync + Send + 'static + DependOnDatabaseConnection {
    type UserEventHandler: UserEventHandler<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn user_event_handler(&self) -> &Self::UserEventHandler;
}
