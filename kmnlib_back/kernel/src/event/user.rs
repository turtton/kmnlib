use serde::{Deserialize, Serialize};

use crate::command::UserCommand;
use crate::entity::{EventVersion, User, UserId, UserName, UserRentLimit};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
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
    pub fn convert(command: UserCommand) -> (UserId, Option<EventVersion<User>>, Self) {
        match command {
            UserCommand::Create {
                id,
                name,
                rent_limit,
            } => {
                let event = Self::Created { name, rent_limit };
                (id, None, event)
            }
            UserCommand::Update {
                id,
                name,
                rent_limit,
            } => {
                let event = Self::Updated { name, rent_limit };
                (id, None, event)
            }
            UserCommand::Delete { id } => {
                let event = Self::Deleted;
                (id, None, event)
            }
        }
    }
}
