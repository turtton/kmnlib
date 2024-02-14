use destructure::Destructure;
use error_stack::Report;

use crate::command::UserCommand;
use crate::entity::{EventVersion, User, UserId, UserName, UserRentLimit};
use crate::event::{Applier, DestructEventInfo, EventInfo, EventRowFieldAttachments};
use crate::KernelError;

const USER_CREATED: &str = "user_created";
const USER_UPDATED: &str = "user_updated";
const USER_DELETED: &str = "user_deleted";

#[derive(Debug, Eq, PartialEq)]
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

impl Applier<EventInfo<UserEvent, User>, UserId> for Option<User> {
    fn apply(&mut self, event: EventInfo<UserEvent, User>, id: UserId) {
        let DestructEventInfo { event, version, .. } = event.into_destruct();
        match (self, event) {
            (option @ None, UserEvent::Created { name, rent_limit }) => {
                *option = Some(User::new(id, name, rent_limit, version));
            }
            (Some(user), UserEvent::Updated { name, rent_limit }) => {
                user.substitute(|user| {
                    if let Some(name) = name {
                        *user.name = name;
                    }
                    if let Some(rent_limit) = rent_limit {
                        *user.rent_limit = rent_limit;
                    }
                    *user.version = version;
                });
            }
            (option, UserEvent::Deleted) => {
                *option = None;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Destructure)]
pub struct UserEventRow {
    event_name: String,
    name: Option<UserName>,
    rent_limit: Option<UserRentLimit>,
}

impl UserEventRow {
    pub fn new(
        event_name: String,
        name: Option<UserName>,
        rent_limit: Option<UserRentLimit>,
    ) -> Self {
        Self {
            event_name,
            name,
            rent_limit,
        }
    }
}

impl From<UserEvent> for UserEventRow {
    fn from(value: UserEvent) -> Self {
        match value {
            UserEvent::Created { name, rent_limit } => {
                Self::new(String::from(USER_CREATED), Some(name), Some(rent_limit))
            }
            UserEvent::Updated { name, rent_limit } => {
                Self::new(String::from(USER_UPDATED), name, rent_limit)
            }
            UserEvent::Deleted => Self::new(String::from(USER_DELETED), None, None),
        }
    }
}

impl TryFrom<UserEventRow> for UserEvent {
    type Error = Report<KernelError>;
    fn try_from(value: UserEventRow) -> Result<Self, Self::Error> {
        let event_name = value.event_name;
        match &*event_name {
            USER_CREATED => {
                let name = value.name.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "name")
                })?;
                let rent_limit = value.rent_limit.ok_or_else(|| {
                    Report::new(KernelError::Internal)
                        .attach_field_details(&event_name, "rent_limit")
                })?;
                Ok(Self::Created { name, rent_limit })
            }
            USER_UPDATED => {
                let name = value.name;
                let rent_limit = value.rent_limit;
                Ok(Self::Updated { name, rent_limit })
            }
            USER_DELETED => Ok(Self::Deleted),
            _ => Err(Report::new(KernelError::Internal).attach_unknown_event("user", &event_name)),
        }
    }
}
