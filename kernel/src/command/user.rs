use crate::entity::{UserId, UserName};
use crate::KernelError;
use error_stack::Report;
use uuid::Uuid;

pub enum UserCommand {
    CreateUser { name: UserName },
    UpdateUserName { id: UserId, name: UserName },
    DeleteUser { id: UserId },
}

#[async_trait::async_trait]
pub trait UserCommandHandler: Sync + Send + 'static {
    async fn handle(&self, command: UserCommand) -> Result<Uuid, Report<KernelError>>;
}
