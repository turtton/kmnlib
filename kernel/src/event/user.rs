use crate::command::UserCommand;
use crate::entity::{EventVersion, User, UserId, UserName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UserEvent {
    Created { name: UserName },
    Updated { name: Option<UserName> },
    Deleted,
}

impl UserEvent {
    pub fn convert(command: UserCommand) -> (String, UserId, Option<EventVersion<User>>, Self) {
        match command {
            UserCommand::Create { id, name } => {
                let event = Self::Created { name };
                ("created-user".to_string(), id, None, event)
            }
            UserCommand::Update { id, name } => {
                let event = Self::Updated { name };
                ("updated-user".to_string(), id, None, event)
            }
            UserCommand::Delete { id } => {
                let event = Self::Deleted;
                ("deleted-user".to_string(), id, None, event)
            }
        }
    }
}
