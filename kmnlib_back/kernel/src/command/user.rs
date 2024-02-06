use crate::database::Transaction;
use crate::entity::{UserId, UserName, UserRentLimit};
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
pub trait UserCommandHandler<Connection: Transaction>: Sync + Send + 'static {
    async fn handle(
        &self,
        con: &mut Connection,
        command: UserCommand,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnUserCommandHandler<Connection: Transaction>: Sync + Send + 'static {
    type UserCommandHandler: UserCommandHandler<Connection>;
    fn user_command_handler(&self) -> &Self::UserCommandHandler;
}
