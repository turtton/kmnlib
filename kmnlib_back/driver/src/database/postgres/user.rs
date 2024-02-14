use error_stack::Report;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use time::OffsetDateTime;

use kernel::interface::command::{UserCommand, UserCommandHandler};
use kernel::interface::event::{DestructUserEventRow, EventInfo, UserEvent, UserEventRow};
use kernel::interface::query::{UserEventQuery, UserQuery};
use kernel::interface::update::UserModifier;
use kernel::prelude::entity::{CreatedAt, EventVersion, User, UserId, UserName, UserRentLimit};
use kernel::KernelError;

use crate::database::postgres::PostgresConnection;
use crate::error::ConvertError;

pub struct PostgresUserRepository;

#[async_trait::async_trait]
impl UserQuery<PostgresConnection> for PostgresUserRepository {
    async fn find_by_id(
        &self,
        con: &mut PostgresConnection,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError> {
        PgUserInternal::find_by_id(con, id).await
    }
}

#[async_trait::async_trait]
impl UserModifier<PostgresConnection> for PostgresUserRepository {
    async fn create(
        &self,
        con: &mut PostgresConnection,
        user: User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::create(con, user).await
    }

    async fn update(
        &self,
        con: &mut PostgresConnection,
        user: User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::update(con, user).await
    }

    async fn delete(
        &self,
        con: &mut PostgresConnection,
        user_id: UserId,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::delete(con, user_id).await
    }
}

#[async_trait::async_trait]
impl UserCommandHandler<PostgresConnection> for PostgresUserRepository {
    async fn handle(
        &self,
        con: &mut PostgresConnection,
        command: UserCommand,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::handle_command(con, command).await
    }
}

#[async_trait::async_trait]
impl UserEventQuery<PostgresConnection> for PostgresUserRepository {
    async fn get_events(
        &self,
        con: &mut PostgresConnection,
        id: &UserId,
        since: Option<&EventVersion<User>>,
    ) -> error_stack::Result<Vec<EventInfo<UserEvent, User>>, KernelError> {
        PgUserInternal::get_events(con, id, since).await
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    rent_limit: i32,
    version: i64,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User::new(
            UserId::new(row.id),
            UserName::new(row.name),
            UserRentLimit::new(row.rent_limit),
            EventVersion::new(row.version),
        )
    }
}

#[derive(sqlx::FromRow)]
struct UserEventRowColumn {
    version: i64,
    event_name: String,
    name: Option<String>,
    rent_limit: Option<i32>,
    created_at: OffsetDateTime,
}

impl TryFrom<UserEventRowColumn> for EventInfo<UserEvent, User> {
    type Error = Report<KernelError>;
    fn try_from(value: UserEventRowColumn) -> Result<Self, Self::Error> {
        let row = UserEventRow::new(
            value.event_name,
            value.name.map(UserName::new),
            value.rent_limit.map(UserRentLimit::new),
        );
        let event = UserEvent::try_from(row)?;
        Ok(EventInfo::new(
            event,
            EventVersion::new(value.version),
            CreatedAt::new(value.created_at),
        ))
    }
}

pub(in crate::database) struct PgUserInternal;

impl PgUserInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError> {
        let row = sqlx::query_as::<_, UserRow>(
            // language=postgresql
            r#"
            SELECT id, name, rent_limit, version
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(con)
        .await
        .convert_error()?;
        let found = row.map(User::from);
        Ok(found)
    }

    async fn create(con: &mut PgConnection, user: User) -> error_stack::Result<(), KernelError> {
        sqlx::query(
            // language=postgresql
            r#"
            INSERT INTO users (id, name, rent_limit, version)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user.id().as_ref())
        .bind(user.name().as_ref())
        .bind(user.rent_limit().as_ref())
        .bind(user.version().as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, user: User) -> error_stack::Result<(), KernelError> {
        // language=postgresql
        sqlx::query(
            r#"
            UPDATE users
            SET name = $2, version = $3
            WHERE id = $1
            "#,
        )
        .bind(user.id().as_ref())
        .bind(user.name().as_ref())
        .bind(user.version().as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn delete(
        con: &mut PgConnection,
        user_id: UserId,
    ) -> error_stack::Result<(), KernelError> {
        // language=postgresql
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id.as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn handle_command(
        con: &mut PgConnection,
        command: UserCommand,
    ) -> error_stack::Result<(), KernelError> {
        let (user_id, event_version, event) = UserEvent::convert(command);
        let DestructUserEventRow {
            event_name,
            name,
            rent_limit,
        } = UserEventRow::from(event).into_destruct();
        let name = name.as_ref().map(AsRef::as_ref);
        let rent_limit = rent_limit.as_ref().map(AsRef::as_ref);
        match event_version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO user_events (user_id, event_name, name, rent_limit) VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(user_id.as_ref())
                .bind(event_name)
                .bind(name)
                .bind(rent_limit)
                .execute(con)
                .await
                .convert_error()?;
            }
            Some(version) => {
                let mut version = version;
                if let EventVersion::Nothing = version {
                    let event = PgUserInternal::get_events(con, &user_id, None).await?;
                    if !event.is_empty() {
                        return Err(Report::new(KernelError::Concurrency)
                            .attach_printable("Event stream is already exists"));
                    } else {
                        version = EventVersion::new(1);
                    }
                }
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO user_events (version, user_id, event_name, name, rent_limit) VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(version.as_ref())
                .execute(con)
                .await
                .convert_error()?;
            }
        }
        Ok(())
    }

    async fn get_events(
        con: &mut PgConnection,
        id: &UserId,
        since: Option<&EventVersion<User>>,
    ) -> error_stack::Result<Vec<EventInfo<UserEvent, User>>, KernelError> {
        let row = match since {
            None => {
                // language=postgresql
                sqlx::query_as::<_, UserEventRowColumn>(
                    r#"
                    SELECT version, event_name, name, rent_limit, created_at
                    FROM user_events
                    WHERE user_id = $1
                    "#,
                )
            }
            Some(version) => {
                // language=postgresql
                sqlx::query_as::<_, UserEventRowColumn>(
                    r#"
                    SELECT version, event_name, name, rent_limit, created_at
                    FROM user_events
                    WHERE version > $1 AND user_id = $2
                    "#,
                )
                .bind(version.as_ref())
            }
        }
        .bind(id.as_ref())
        .fetch_all(con)
        .await
        .convert_error()?;

        row.into_iter().map(EventInfo::try_from).collect()
    }
}

#[cfg(test)]
mod test {
    use error_stack::ResultExt;
    use uuid::Uuid;

    use kernel::interface::command::{UserCommand, UserCommandHandler};
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::event::UserEvent;
    use kernel::interface::query::{UserEventQuery, UserQuery};
    use kernel::interface::update::UserModifier;
    use kernel::prelude::entity::{EventVersion, User, UserId, UserName, UserRentLimit};
    use kernel::KernelError;

    use crate::database::postgres::user::PostgresUserRepository;
    use crate::database::postgres::PostgresDatabase;

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn find_by_id() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new()
            .await
            .change_context_lazy(|| KernelError::Internal)?;
        let mut connection = db.transact().await?;
        let id = UserId::new(Uuid::new_v4());
        let user = User::new(
            id.clone(),
            UserName::new("test".to_string()),
            UserRentLimit::new(1),
            EventVersion::new(0),
        );

        PostgresUserRepository
            .create(&mut connection, user.clone())
            .await?;

        let found = PostgresUserRepository
            .find_by_id(&mut connection, &id)
            .await?;
        assert_eq!(found, Some(user.clone()));

        let user = user.reconstruct(|u| u.name = UserName::new("test2".to_string()));
        PostgresUserRepository
            .update(&mut connection, user.clone())
            .await?;

        let found = PostgresUserRepository
            .find_by_id(&mut connection, &id)
            .await?;
        assert_eq!(found, Some(user));

        PostgresUserRepository
            .delete(&mut connection, id.clone())
            .await?;
        let found = PostgresUserRepository
            .find_by_id(&mut connection, &id)
            .await?;
        assert!(found.is_none());

        Ok(())
    }

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn test_event() -> error_stack::Result<(), KernelError> {
        let db = PostgresDatabase::new().await?;
        let mut connection = db.transact().await?;
        let id = UserId::new(Uuid::new_v4());
        let name = UserName::new("test".to_string());
        let rent_limit = UserRentLimit::new(1);

        let create_command = UserCommand::Create {
            id: id.clone(),
            name,
            rent_limit,
        };
        PostgresUserRepository
            .handle(&mut connection, create_command.clone())
            .await?;
        let create_event = PostgresUserRepository
            .get_events(&mut connection, &id, None)
            .await?;
        let create_event = create_event.first().unwrap();
        let event_version_first = EventVersion::new(1);
        assert_eq!(create_event.version(), &event_version_first);
        let (_, _, expected_event) = UserEvent::convert(create_command);
        assert_eq!(create_event.event(), &expected_event);

        let update_command = UserCommand::Update {
            id: id.clone(),
            name: Some(UserName::new("test2".to_string())),
            rent_limit: None,
        };
        PostgresUserRepository
            .handle(&mut connection, update_command.clone())
            .await?;
        let update_event = PostgresUserRepository
            .get_events(&mut connection, &id, Some(&event_version_first))
            .await?;
        let update_event = update_event.first().unwrap();
        assert_eq!(update_event.version(), &EventVersion::new(2));
        let (_, _, expected_event) = UserEvent::convert(update_command);
        assert_eq!(update_event.event(), &expected_event);

        // TODO: create user entity
        Ok(())
    }
}
