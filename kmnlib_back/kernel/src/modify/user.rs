use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::{User, UserId};
use crate::KernelError;

#[async_trait::async_trait]
pub trait UserModifier: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn create(
        &self,
        con: &mut Self::Transaction,
        user: &User,
    ) -> error_stack::Result<(), KernelError>;
    async fn update(
        &self,
        con: &mut Self::Transaction,
        user: &User,
    ) -> error_stack::Result<(), KernelError>;
    async fn delete(
        &self,
        con: &mut Self::Transaction,
        user_id: &UserId,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnUserModifier: 'static + Sync + Send + DependOnDatabaseConnection {
    type UserModifier: UserModifier<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn user_modifier(&self) -> &Self::UserModifier;
}
