use crate::error::DriverError;
use error_stack::{Report, ResultExt};
use kernel::interface::query::UserQuery;
use kernel::interface::update::UserModifier;
use kernel::prelude::entity::{EventVersion, User, UserId, UserName};
use sqlx::pool::PoolConnection;
use sqlx::types::Uuid;
use sqlx::{PgConnection, Postgres};

pub struct PostgresUserQuery;

#[async_trait::async_trait]
impl UserQuery<PoolConnection<Postgres>> for PostgresUserQuery {
    type Error = DriverError;
    async fn find_by_id(
        &self,
        con: &mut PoolConnection<Postgres>,
        id: &UserId,
    ) -> Result<Option<User>, Report<DriverError>> {
        PgUserInternal::find_by_id(con, id).await
    }
}

#[async_trait::async_trait]
impl UserModifier<PoolConnection<Postgres>> for PostgresUserQuery {
    type Error = DriverError;

    async fn create(
        &self,
        con: &mut PoolConnection<Postgres>,
        user: User,
    ) -> Result<(), Report<DriverError>> {
        PgUserInternal::create(con, user).await
    }

    async fn update(
        &self,
        con: &mut PoolConnection<Postgres>,
        user: User,
    ) -> Result<(), Report<DriverError>> {
        PgUserInternal::update(con, user).await
    }

    async fn delete(
        &self,
        con: &mut PoolConnection<Postgres>,
        user_id: UserId,
    ) -> Result<(), Report<DriverError>> {
        PgUserInternal::delete(con, user_id).await
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    version: i64,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User::new(
            UserId::new(row.id),
            UserName::new(row.name),
            EventVersion::new(row.version),
        )
    }
}

pub(in crate::database) struct PgUserInternal;

impl PgUserInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        id: &UserId,
    ) -> Result<Option<User>, Report<DriverError>> {
        let row = sqlx::query_as::<_, UserRow>(
            // language=postgresql
            r#"
            SELECT id, name, version
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        let found = row.map(User::from);
        Ok(found)
    }

    async fn create(con: &mut PgConnection, user: User) -> Result<(), Report<DriverError>> {
        sqlx::query(
            // language=postgresql
            r#"
            INSERT INTO users (id, name, version)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user.id().as_ref())
        .bind(user.name().as_ref())
        .bind(user.version().as_ref())
        .execute(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }

    async fn update(con: &mut PgConnection, user: User) -> Result<(), Report<DriverError>> {
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
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }

    async fn delete(con: &mut PgConnection, user_id: UserId) -> Result<(), Report<DriverError>> {
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
        .change_context_lazy(|| DriverError::SqlX)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::database::postgres::user::PostgresUserQuery;
    use crate::database::postgres::PostgresDatabase;
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::UserQuery;
    use kernel::interface::update::UserModifier;
    use kernel::prelude::entity::{EventVersion, User, UserId, UserName};
    use uuid::Uuid;

    #[test_with::env(POSTGRES_TEST)]
    #[tokio::test]
    async fn find_by_id() -> Result<(), Report<DriverError>> {
        let db = PostgresDatabase::new().await?;
        let mut connection = db.transact().await?;
        let id = UserId::new(Uuid::new_v4());
        let user = User::new(
            id.clone(),
            UserName::new("test".to_string()),
            EventVersion::new(0),
        );

        PostgresUserQuery
            .create(&mut connection, user.clone())
            .await?;

        let found = PostgresUserQuery.find_by_id(&mut connection, &id).await?;
        assert_eq!(found, Some(user.clone()));

        let user = user.reconstruct(|u| u.name = UserName::new("test2".to_string()));
        PostgresUserQuery
            .update(&mut connection, user.clone())
            .await?;

        let found = PostgresUserQuery.find_by_id(&mut connection, &id).await?;
        assert_eq!(found, Some(user));

        PostgresUserQuery
            .delete(&mut connection, id.clone())
            .await?;
        let found = PostgresUserQuery.find_by_id(&mut connection, &id).await?;
        assert!(found.is_none());

        Ok(())
    }
}
