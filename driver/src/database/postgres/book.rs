use crate::error::DriverError;
use error_stack::{Report, ResultExt};
use kernel::interface::query::BookQuery;
use kernel::prelude::entity::{Book, BookId, BookTitle, EventVersion};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use uuid::Uuid;

pub struct PostgresBookQuery;

#[async_trait::async_trait]
impl BookQuery<PoolConnection<Postgres>> for PostgresBookQuery {
    type Error = DriverError;
    async fn find_by_id(
        &self,
        con: &mut PoolConnection<Postgres>,
        id: &BookId,
    ) -> Result<Option<Book>, Report<Self::Error>> {
        PgBookInternal::find_by_id(con, id).await
    }
}

#[derive(sqlx::FromRow)]
struct BookRow {
    id: Uuid,
    title: String,
    rev_id: i64,
}

impl From<BookRow> for Book {
    fn from(value: BookRow) -> Self {
        Book::new(
            BookId::new(value.id),
            BookTitle::new(value.title),
            EventVersion::new(value.rev_id),
        )
    }
}

pub(in crate::database) struct PgBookInternal;

impl PgBookInternal {
    async fn find_by_id(
        con: &mut PgConnection,
        id: &BookId,
    ) -> Result<Option<Book>, Report<DriverError>> {
        let row = sqlx::query_as::<_, BookRow>(
            r#"
            SELECT id, title
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(con)
        .await
        .change_context_lazy(|| DriverError::SqlX)?;
        let found = row.map(Book::from);
        Ok(found)
    }
}

#[cfg(test)]
mod test {
    use crate::database::postgres::book::PostgresBookQuery;
    use crate::database::postgres::PostgresDatabase;
    use crate::error::DriverError;
    use error_stack::Report;
    use kernel::interface::database::QueryDatabaseConnection;
    use kernel::interface::query::BookQuery;
    use kernel::prelude::entity::BookId;

    #[test_with::env(POSTGRES)]
    #[tokio::test]
    async fn find_by_id() -> Result<(), Report<DriverError>> {
        let db = PostgresDatabase::new().await?;
        let mut con = db.transact().await?;
        let id = BookId::new(uuid::Uuid::new_v4());
        let found = PostgresBookQuery.find_by_id(&mut con, &id).await?;
        assert!(found.is_some());
        Ok(())
    }
}
