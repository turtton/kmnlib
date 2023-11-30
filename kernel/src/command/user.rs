use crate::entity::{UserId, UserName};
use error_stack::{Context, Report};
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum UserCommand {
    CreateUser { name: UserName },
    UpdateUserName { id: UserId, name: UserName },
    DeleteUser { id: UserId },
}

#[async_trait::async_trait]
pub trait UserCommandHandler: Sync + Send + 'static {
    type Error: Context;
    async fn handle(&self, command: UserCommand) -> Result<Uuid, Report<Self::Error>>;
}

pub trait DependOnUserCommandHandler: Sync + Send + 'static {
    type UserCommandHandler: UserCommandHandler;
    fn user_command_handler(&self) -> &Self::UserCommandHandler;
}
