use crate::transfer::UserDto;
use error_stack::Report;
use kernel::interface::database::{DependOnDatabaseConnection, QueryDatabaseConnection};
use kernel::interface::event::UserEvent;
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
    async fn get_user(&self, id: Uuid) -> error_stack::Result<Option<UserDto>, KernelError> {
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
                    let first = user_events.remove(0);
                    match first {
                        UserEvent::Created { name, rent_limit } => {
                            Some(User::new(id, name, rent_limit, EventVersion::new(0)))
                        }
                        _ => Err(Report::new(KernelError::Internal).attach_printable(format!(
                            "User first event is {first:?} instead of UserEvent::Created"
                        )))?,
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
                    match event {
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
                Ok(Some(user.try_into()?))
            }
        }
    }
}
