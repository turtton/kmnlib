use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{BookId, Rent, UserId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait RentModifier: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn create(
        &self,
        con: &mut Self::Transaction,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError>;

    async fn update(
        &self,
        con: &mut Self::Transaction,
        rent: &Rent,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Self::Transaction,
        book_id: &BookId,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnRentModifier: 'static + Sync + Send + DependOnDatabaseConnection {
    type RentModifier: RentModifier<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn rent_modifier(&self) -> &Self::RentModifier;
}
