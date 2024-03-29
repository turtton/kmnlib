use error_stack::Report;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use time::OffsetDateTime;

use kernel::interface::event::{
    CommandInfo, DestructCommandInfo, DestructUserEventRow, EventInfo, UserEvent, UserEventRow,
};
use kernel::interface::query::{
    DependOnUserEventQuery, DependOnUserQuery, UserEventQuery, UserQuery,
};
use kernel::interface::update::{
    DependOnUserEventHandler, DependOnUserModifier, UserEventHandler, UserModifier,
};
use kernel::prelude::entity::{
    CreatedAt, EventVersion, ExpectedEventVersion, IsDeleted, SelectLimit, SelectOffset, User,
    UserId, UserName, UserRentLimit,
};
use kernel::KernelError;

use crate::database::postgres::PostgresTransaction;
use crate::database::PostgresDatabase;
use crate::error::ConvertError;

pub struct PostgresUserRepository;

#[async_trait::async_trait]
impl UserQuery for PostgresUserRepository {
    type Transaction = PostgresTransaction;

    async fn get_all(
        &self,
        con: &mut Self::Transaction,
        limit: &SelectLimit,
        offset: &SelectOffset,
    ) -> error_stack::Result<Vec<User>, KernelError> {
        PgUserInternal::get_all(con, limit, offset).await
    }

    async fn find_by_id(
        &self,
        con: &mut PostgresTransaction,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError> {
        PgUserInternal::find_by_id(con, id).await
    }
}

impl DependOnUserQuery for PostgresDatabase {
    type UserQuery = PostgresUserRepository;
    fn user_query(&self) -> &Self::UserQuery {
        &PostgresUserRepository
    }
}

#[async_trait::async_trait]
impl UserModifier for PostgresUserRepository {
    type Transaction = PostgresTransaction;
    async fn create(
        &self,
        con: &mut PostgresTransaction,
        user: &User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::create(con, user).await
    }

    async fn update(
        &self,
        con: &mut PostgresTransaction,
        user: &User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::update(con, user).await
    }

    async fn delete(
        &self,
        con: &mut PostgresTransaction,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::delete(con, user_id).await
    }
}

impl DependOnUserModifier for PostgresDatabase {
    type UserModifier = PostgresUserRepository;
    fn user_modifier(&self) -> &Self::UserModifier {
        &PostgresUserRepository
    }
}

#[async_trait::async_trait]
impl UserEventHandler for PostgresUserRepository {
    type Transaction = PostgresTransaction;
    async fn handle(
        &self,
        con: &mut PostgresTransaction,
        command: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<UserId, KernelError> {
        PgUserInternal::handle_command(con, command).await
    }
}

impl DependOnUserEventHandler for PostgresDatabase {
    type UserEventHandler = PostgresUserRepository;
    fn user_event_handler(&self) -> &Self::UserEventHandler {
        &PostgresUserRepository
    }
}

#[async_trait::async_trait]
impl UserEventQuery for PostgresUserRepository {
    type Transaction = PostgresTransaction;
    async fn get_events(
        &self,
        con: &mut PostgresTransaction,
        id: &UserId,
        since: Option<&EventVersion<User>>,
    ) -> error_stack::Result<Vec<EventInfo<UserEvent, User>>, KernelError> {
        PgUserInternal::get_events(con, id, since).await
    }
}

impl DependOnUserEventQuery for PostgresDatabase {
    type UserEventQuery = PostgresUserRepository;
    fn user_event_query(&self) -> &Self::UserEventQuery {
        &PostgresUserRepository
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    rent_limit: i32,
    version: i64,
    is_deleted: bool,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User::new(
            UserId::new(row.id),
            UserName::new(row.name),
            UserRentLimit::new(row.rent_limit),
            EventVersion::new(row.version),
            IsDeleted::new(row.is_deleted),
        )
    }
}

#[derive(sqlx::FromRow)]
struct UserEventRowColumn {
    version: i64,
    event_name: String,
    user_id: Uuid,
    name: Option<String>,
    rent_limit: Option<i32>,
    created_at: OffsetDateTime,
}

impl TryFrom<UserEventRowColumn> for EventInfo<UserEvent, User> {
    type Error = Report<KernelError>;
    fn try_from(value: UserEventRowColumn) -> Result<Self, Self::Error> {
        let row = UserEventRow::new(
            value.event_name,
            UserId::new(value.user_id),
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
    async fn get_all(
        con: &mut PgConnection,
        limit: &SelectLimit,
        offset: &SelectOffset,
    ) -> error_stack::Result<Vec<User>, KernelError> {
        sqlx::query_as::<_, UserRow>(
            //language=postgresql
            r#"
            SELECT id, name, rent_limit, version
            FROM users
            ORDER BY id
            LIMIT $1
            OFFSET $2
            "#,
        )
        .bind(limit.as_ref())
        .bind(offset.as_ref())
        .fetch_all(con)
        .await
        .convert_error()
        .map(|vec| vec.into_iter().map(User::from).collect())
    }
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

    async fn create(con: &mut PgConnection, user: &User) -> error_stack::Result<(), KernelError> {
        sqlx::query(
            // language=postgresql
            r#"
            INSERT INTO users (id, name, rent_limit, version, is_deleted)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user.id().as_ref())
        .bind(user.name().as_ref())
        .bind(user.rent_limit().as_ref())
        .bind(user.version().as_ref())
        .bind(user.is_deleted().as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, user: &User) -> error_stack::Result<(), KernelError> {
        // language=postgresql
        sqlx::query(
            r#"
            UPDATE users
            SET name = $2, version = $3, is_deleted = $4
            WHERE id = $1
            "#,
        )
        .bind(user.id().as_ref())
        .bind(user.name().as_ref())
        .bind(user.version().as_ref())
        .bind(user.is_deleted().as_ref())
        .execute(con)
        .await
        .convert_error()?;
        Ok(())
    }

    async fn delete(
        con: &mut PgConnection,
        user_id: &UserId,
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
        command: CommandInfo<UserEvent, User>,
    ) -> error_stack::Result<UserId, KernelError> {
        let DestructCommandInfo { event, version } = command.into_destruct();
        let DestructUserEventRow {
            event_name,
            id,
            name,
            rent_limit,
        } = UserEventRow::from(event).into_destruct();
        let name = name.as_ref().map(AsRef::as_ref);
        let rent_limit = rent_limit.as_ref().map(AsRef::as_ref);
        match version {
            None => {
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO user_events (user_id, event_name, name, rent_limit) VALUES ($1, $2, $3, $4)
                    "#,
                )
                    .bind(id.as_ref())
                    .bind(event_name)
                    .bind(name)
                    .bind(rent_limit)
                    .execute(con)
                    .await
                    .convert_error()?;
            }
            Some(version) => {
                let version = match version {
                    ExpectedEventVersion::Nothing => {
                        let event = PgUserInternal::get_events(con, &id, None).await?;
                        if !event.is_empty() {
                            return Err(Report::new(KernelError::Concurrency)
                                .attach_printable("Event stream is already exists"));
                        } else {
                            EventVersion::new(1)
                        }
                    }
                    ExpectedEventVersion::Exact(version) => version,
                };
                // language=postgresql
                sqlx::query(
                    r#"
                    INSERT INTO user_events (version, user_id, event_name, name, rent_limit) VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                    .bind(version.as_ref())
                    .bind(id.as_ref())
                    .bind(event_name)
                    .bind(name)
                    .bind(rent_limit)
                    .execute(con)
                    .await
                    .convert_error()?;
            }
        }
        Ok(id)
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

    use kernel::interface::database::DatabaseConnection;
    use kernel::interface::event::{CommandInfo, UserEvent};
    use kernel::interface::query::{UserEventQuery, UserQuery};
    use kernel::interface::update::{UserEventHandler, UserModifier};
    use kernel::prelude::entity::{EventVersion, IsDeleted, User, UserId, UserName, UserRentLimit};
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
            IsDeleted::new(false),
        );

        PostgresUserRepository
            .create(&mut connection, &user)
            .await?;

        let found = PostgresUserRepository
            .find_by_id(&mut connection, &id)
            .await?;
        assert_eq!(found, Some(user.clone()));

        let user = user.reconstruct(|u| u.name = UserName::new("test2".to_string()));
        PostgresUserRepository
            .update(&mut connection, &user)
            .await?;

        let found = PostgresUserRepository
            .find_by_id(&mut connection, &id)
            .await?;
        assert_eq!(found, Some(user));

        PostgresUserRepository.delete(&mut connection, &id).await?;
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

        let create_event = UserEvent::Create {
            id: id.clone(),
            name,
            rent_limit,
        };
        let create_command = CommandInfo::new(create_event, None);
        PostgresUserRepository
            .handle(&mut connection, create_command.clone())
            .await?;
        let create_event = PostgresUserRepository
            .get_events(&mut connection, &id, None)
            .await?;
        let create_event = create_event.first().unwrap();
        let event_version_first = EventVersion::new(1);
        assert_eq!(create_event.version(), &event_version_first);
        assert_eq!(create_event.event(), &create_command.into_destruct().event);

        let update_event = UserEvent::Update {
            id: id.clone(),
            name: Some(UserName::new("test2".to_string())),
            rent_limit: None,
        };
        let update_command = CommandInfo::new(update_event, None);
        PostgresUserRepository
            .handle(&mut connection, update_command.clone())
            .await?;
        let update_event = PostgresUserRepository
            .get_events(&mut connection, &id, Some(&event_version_first))
            .await?;
        let update_event = update_event.first().unwrap();
        assert_eq!(update_event.version(), &EventVersion::new(2));
        assert_eq!(update_event.event(), &update_command.into_destruct().event);

        // TODO: create user entity
        Ok(())
    }
}
