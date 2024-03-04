use destructure::Destructure;
use error_stack::Report;
use serde::{Deserialize, Serialize};

use crate::entity::{IsDeleted, User, UserId, UserName, UserRentLimit};
use crate::event::{Applier, DestructEventInfo, EventInfo, EventRowFieldAttachments};
use crate::KernelError;

const USER_CREATED: &str = "user_created";
const USER_UPDATED: &str = "user_updated";
const USER_DELETED: &str = "user_deleted";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum UserEvent {
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

impl Applier<EventInfo<UserEvent, User>> for User {
    fn apply(&mut self, event: EventInfo<UserEvent, User>) {
        let DestructEventInfo { event, version, .. } = event.into_destruct();
        match event {
            UserEvent::Create { .. } => {}
            UserEvent::Update {
                name, rent_limit, ..
            } => self.substitute(|user| {
                if let Some(name) = name {
                    *user.name = name;
                }
                if let Some(rent_limit) = rent_limit {
                    *user.rent_limit = rent_limit;
                }
                *user.version = version;
            }),
            UserEvent::Delete { .. } => {
                self.substitute(|user| *user.is_deleted = IsDeleted::new(true))
            }
        }
    }
}

impl Applier<EventInfo<UserEvent, User>> for Option<User> {
    fn apply(&mut self, event: EventInfo<UserEvent, User>) {
        let DestructEventInfo { event, version, .. } = event.into_destruct();
        match (self, event) {
            (
                option @ None,
                UserEvent::Create {
                    id,
                    name,
                    rent_limit,
                },
            ) => {
                *option = Some(User::new(
                    id,
                    name,
                    rent_limit,
                    version,
                    IsDeleted::new(false),
                ))
            }
            (
                Some(user),
                UserEvent::Update {
                    name, rent_limit, ..
                },
            ) => user.substitute(|user| {
                if let Some(name) = name {
                    *user.name = name;
                }
                if let Some(rent_limit) = rent_limit {
                    *user.rent_limit = rent_limit;
                }
                *user.version = version;
            }),
            (Some(user), UserEvent::Delete { .. }) => {
                user.substitute(|user| *user.is_deleted = IsDeleted::new(true))
            }
            _ => {}
        }
    }
}

#[derive(Debug, Destructure)]
pub struct UserEventRow {
    event_name: String,
    id: UserId,
    name: Option<UserName>,
    rent_limit: Option<UserRentLimit>,
}

impl UserEventRow {
    pub fn new(
        event_name: String,
        id: UserId,
        name: Option<UserName>,
        rent_limit: Option<UserRentLimit>,
    ) -> Self {
        Self {
            event_name,
            id,
            name,
            rent_limit,
        }
    }
}

impl From<UserEvent> for UserEventRow {
    fn from(value: UserEvent) -> Self {
        match value {
            UserEvent::Create {
                id,
                name,
                rent_limit,
            } => Self::new(String::from(USER_CREATED), id, Some(name), Some(rent_limit)),
            UserEvent::Update {
                id,
                name,
                rent_limit,
            } => Self::new(String::from(USER_UPDATED), id, name, rent_limit),
            UserEvent::Delete { id } => Self::new(String::from(USER_DELETED), id, None, None),
        }
    }
}

impl TryFrom<UserEventRow> for UserEvent {
    type Error = Report<KernelError>;
    fn try_from(value: UserEventRow) -> Result<Self, Self::Error> {
        let event_name = value.event_name;
        match &*event_name {
            USER_CREATED => {
                let id = value.id;
                let name = value.name.ok_or_else(|| {
                    Report::new(KernelError::Internal).attach_field_details(&event_name, "name")
                })?;
                let rent_limit = value.rent_limit.ok_or_else(|| {
                    Report::new(KernelError::Internal)
                        .attach_field_details(&event_name, "rent_limit")
                })?;
                Ok(Self::Create {
                    id,
                    name,
                    rent_limit,
                })
            }
            USER_UPDATED => {
                let id = value.id;
                let name = value.name;
                let rent_limit = value.rent_limit;
                Ok(Self::Update {
                    id,
                    name,
                    rent_limit,
                })
            }
            USER_DELETED => Ok(Self::Delete { id: value.id }),
            _ => Err(Report::new(KernelError::Internal).attach_unknown_event("user", &event_name)),
        }
    }
}
