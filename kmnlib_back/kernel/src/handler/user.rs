use crate::database::Transaction;
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
pub trait UserEventHandler<Connection: Transaction>: Sync + Send + 'static {
    async fn handle(
        &self,
        con: &mut Connection,
        event: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnUserEventHandler<Connection: Transaction>: Sync + Send + 'static {
    type UserEventHandler: UserEventHandler<Connection>;
    fn user_event_handler(&self) -> &Self::UserEventHandler;
}
