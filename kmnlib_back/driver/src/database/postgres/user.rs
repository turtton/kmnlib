use sqlx::pool::PoolConnection;
use sqlx::types::Uuid;
use sqlx::{PgConnection, Postgres};

use kernel::interface::query::UserQuery;
use kernel::interface::update::UserModifier;
use kernel::prelude::entity::{EventVersion, User, UserId, UserName, UserRentLimit};
use kernel::KernelError;

use crate::error::ConvertError;

pub struct PostgresUserRepository;

#[async_trait::async_trait]
impl UserQuery<PoolConnection<Postgres>> for PostgresUserRepository {
    async fn find_by_id(
        &self,
        con: &mut PoolConnection<Postgres>,
        id: &UserId,
    ) -> error_stack::Result<Option<User>, KernelError> {
        PgUserInternal::find_by_id(con, id).await
    }
}

#[async_trait::async_trait]
impl UserModifier<PoolConnection<Postgres>> for PostgresUserRepository {
    async fn create(
        &self,
        con: &mut PoolConnection<Postgres>,
        user: User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::create(con, user).await
    }

    async fn update(
        &self,
        con: &mut PoolConnection<Postgres>,
        user: User,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::update(con, user).await
    }

    async fn delete(
        &self,
        con: &mut PoolConnection<Postgres>,
        user_id: UserId,
    ) -> error_stack::Result<(), KernelError> {
        PgUserInternal::delete(con, user_id).await
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
}

#[cfg(test)]
mod test {
    use error_stack::ResultExt;
    use uuid::Uuid;

    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::UserQuery;
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
}
