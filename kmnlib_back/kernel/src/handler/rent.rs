use crate::database::{DatabaseConnection, DependOnDatabaseConnection, Transaction};
use crate::entity::Rent;
use crate::event::{CommandInfo, RentEvent};
use crate::KernelError;

#[async_trait::async_trait]
pub trait RentEventHandler: 'static + Sync + Send {
    type Transaction: Transaction;
    async fn handle(
        &self,
        con: &mut Self::Transaction,
        event: CommandInfo<RentEvent, Rent>,
    ) -> error_stack::Result<(), KernelError>;
}

pub trait DependOnRentEventHandler: 'static + Sync + Send + DependOnDatabaseConnection {
    type RentEventHandler: RentEventHandler<
        Transaction = <Self::DatabaseConnection as DatabaseConnection>::Transaction,
    >;
    fn rent_event_handler(&self) -> &Self::RentEventHandler;
}
