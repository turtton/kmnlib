use crate::error::DriverError;
use error_stack::{Report, ResultExt};
use kernel::interface::query::UserQuery;
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

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    name: String,
    rev_id: i64,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User::new(
            UserId::new(row.id),
            UserName::new(row.name),
            EventVersion::new(row.rev_id),
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
            r#"
            SELECT id, name
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
}

#[cfg(test)]
mod test {
    use crate::database::postgres::user::PostgresUserQuery;
    use crate::database::postgres::PostgresDatabase;
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::UserQuery;
    use kernel::prelude::entity::UserId;
    use uuid::Uuid;

    #[test_with::env(POSTGRES)]
    #[tokio::test]
    async fn find_by_id() -> Result<(), Report<DriverError>> {
        let db = PostgresDatabase::new().await?;
        let mut connection = db.transact().await?;
        let id = UserId::new(Uuid::new_v4());
        let result = PostgresUserQuery.find_by_id(&mut connection, &id).await?;
        assert!(result.is_some());
        Ok(())
    }
}
