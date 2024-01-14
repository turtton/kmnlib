use crate::command::UserCommand;
use crate::entity::{EventVersion, User, UserId, UserName, UserRentLimit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UserEvent {
    Created {
        name: UserName,
        rent_limit: UserRentLimit,
    },
    Updated {
        name: Option<UserName>,
        rent_limit: Option<UserRentLimit>,
    },
    Deleted,
}

impl UserEvent {
    pub fn convert(command: UserCommand) -> (String, UserId, Option<EventVersion<User>>, Self) {
        match command {
            UserCommand::Create {
                id,
                name,
                rent_limit,
            } => {
                let event = Self::Created { name, rent_limit };
                ("created-user".to_string(), id, None, event)
            }
            UserCommand::Update {
                id,
                name,
                rent_limit,
            } => {
                let event = Self::Updated { name, rent_limit };
                ("updated-user".to_string(), id, None, event)
            }
            UserCommand::Delete { id } => {
                let event = Self::Deleted;
                ("deleted-user".to_string(), id, None, event)
            }
        }
    }
}
