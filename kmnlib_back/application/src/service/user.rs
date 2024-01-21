use crate::transfer::UserDto;
use error_stack::Report;
use kernel::interface::database::{DependOnDatabaseConnection, QueryDatabaseConnection};
use kernel::interface::event::{DestructEventInfo, UserEvent};
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::prelude::entity::{EventVersion, User, UserId};
use kernel::KernelError;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait GetUserService<Connection: Send>:
    'static
    + Sync
    + Send
    + DependOnDatabaseConnection<Connection>
    + DependOnUserQuery<Connection>
    + DependOnUserEventQuery
{
    async fn get_user(&mut self, id: Uuid) -> error_stack::Result<Option<UserDto>, KernelError> {
        let mut connection = self.database_connection().transact().await?;

        let id = UserId::new(id);
        let user = self.user_query().find_by_id(&mut connection, &id).await?;

        let version = user.as_ref().map(|u| u.version());
        let mut user_events = self.user_event_query().get_events(&id, version).await?;

        let user = match user {
            None => {
                if user_events.is_empty() {
                    None
                } else {
                    let first = user_events.remove(0).into_destruct();
                    match first.event {
                        UserEvent::Created { name, rent_limit } => {
                            Some(User::new(id, name, rent_limit, EventVersion::new(0)))
                        }
                        event => {
                            return Err(Report::new(KernelError::Internal).attach_printable(
                                format!(
                                    "User first event is {event:?} instead of UserEvent::Created"
                                ),
                            ))
                        }
                    }
                }
            }
            user => user,
        };
        match user {
            None => Ok(None),
            Some(user) => {
                let mut user = user;
                for event in user_events {
                    match event.into_destruct().event {
                        UserEvent::Created { .. } => {
                            return Err(Report::new(KernelError::Internal).attach_printable(
                                format!(
                                    "Invalid UserEvent::Created ware found in User({:?})",
                                    user.id()
                                ),
                            ))
                        }
                        UserEvent::Updated { name, rent_limit } => {
                            if let Some(name) = name {
                                user = user.reconstruct(|u| u.name = name);
                            }
                            if let Some(rent_limit) = rent_limit {
                                user = user.reconstruct(|u| u.rent_limit = rent_limit)
                            }
                        }
                        UserEvent::Deleted => return Ok(None),
                    }
                }
                // TODO: Modify User entity in other worker
                let user_dto: UserDto = user.try_into()?;
                Ok(Some(user_dto))
            }
        }
    }
}
