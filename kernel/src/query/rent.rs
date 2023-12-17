use crate::entity::{BookId, Rent, UserId};
use error_stack::{Context, Report};

#[async_trait::async_trait]
pub trait RentQuery<Connection>: Sync + Send + 'static {
    type Error: Context;

    async fn find_by_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> Result<Option<Rent>, Report<Self::Error>>;
    async fn find_by_book_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
    ) -> Result<Vec<Rent>, Report<Self::Error>>;

    async fn find_by_user_id(
        &self,
        con: &mut Connection,
        user_id: &UserId,
    ) -> Result<Vec<Rent>, Report<Self::Error>>;
}

pub trait DependOnRentQuery<Connection>: Sync + Send + 'static {
    type RentQuery: RentQuery<Connection>;
    fn rent_query(&self) -> &Self::RentQuery;
}
