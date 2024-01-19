use crate::entity::{BookId, Rent, UserId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait RentModifier<Connection>: 'static + Sync + Send {
    async fn create(
        &self,
        con: &mut Connection,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Connection,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnRentModifier<Connection>: 'static + Sync + Send {
    type RentModifier: RentModifier<Connection>;
    fn rent_modifier(&self) -> &Self::RentModifier;
}
