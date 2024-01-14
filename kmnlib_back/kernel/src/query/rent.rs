use crate::entity::{BookId, Rent, UserId};

#[async_trait::async_trait]
pub trait RentQuery<Connection>: Sync + Send + 'static {
    type Error;

    async fn find_by_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> Result<Option<Rent>, Self::Error>;
    async fn find_by_book_id(
        &self,
        con: &mut Connection,
        book_id: &BookId,
    ) -> Result<Vec<Rent>, Self::Error>;

    async fn find_by_user_id(
        &self,
        con: &mut Connection,
        user_id: &UserId,
    ) -> Result<Vec<Rent>, Self::Error>;
}

pub trait DependOnRentQuery<Connection>: Sync + Send + 'static {
    type RentQuery: RentQuery<Connection>;
    fn rent_query(&self) -> &Self::RentQuery;
}
