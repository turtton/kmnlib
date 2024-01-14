use crate::entity::{BookId, Rent, UserId};

#[async_trait::async_trait]
pub trait RentModifier<Connection>: 'static + Sync + Send {
    type Error;
    async fn create(&self, con: &mut Connection, rent: &Rent) -> Result<(), Self::Error>;
    async fn delete(
        &self,
        con: &mut Connection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> Result<(), Self::Error>;
}

pub trait DependOnRentModifier<Connection>: 'static + Sync + Send {
    type RentModifier: RentModifier<Connection>;
    fn rent_modifier(&self) -> &Self::RentModifier;
}
