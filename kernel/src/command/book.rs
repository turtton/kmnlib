use crate::entity::{Book, BookId, BookTitle, EventNumber};
use crate::KernelError;
use error_stack::Report;
use serde::{Deserialize, Serialize};
use strum::Display;
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Display)]
pub enum BookCommand {
    CreateBook {
        title: BookTitle,
    },
    UpdateBook {
        id: BookId,
        title: BookTitle,
    },
    DeleteBook {
        id: BookId,
    },
}

#[async_trait::async_trait]
pub trait BookCommandHandler {
    async fn handle(&self, command: BookCommand) -> Result<Uuid, Report<KernelError>>;
}
