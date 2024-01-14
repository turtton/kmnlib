use serde::{Deserialize, Serialize};

use crate::entity::{EventVersion, User, UserId, UserName, UserRentLimit};

pub static USER_STREAM_NAME: &str = "user-stream";

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub trait UserCommandHandler: Sync + Send + 'static {
    type Error;
    async fn handle(&self, command: UserCommand) -> Result<EventVersion<User>, Self::Error>;
}

pub trait DependOnUserCommandHandler: Sync + Send + 'static {
    type UserCommandHandler: UserCommandHandler;
    fn user_command_handler(&self) -> &Self::UserCommandHandler;
}
