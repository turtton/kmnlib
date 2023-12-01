use crate::entity::{EventVersion, User, UserId, UserName};
use error_stack::{Context, Report};
use serde::{Deserialize, Serialize};

pub static USER_STREAM_NAME: &str = "user-stream";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserCommand {
    Create { id: UserId, name: UserName },
    UpdateName { id: UserId, name: UserName },
    Delete { id: UserId },
}

#[async_trait::async_trait]
pub trait UserCommandHandler: Sync + Send + 'static {
    type Error: Context;
    async fn handle(&self, command: UserCommand)
        -> Result<EventVersion<User>, Report<Self::Error>>;
}

pub trait DependOnUserCommandHandler: Sync + Send + 'static {
    type UserCommandHandler: UserCommandHandler;
    fn user_command_handler(&self) -> &Self::UserCommandHandler;
}
