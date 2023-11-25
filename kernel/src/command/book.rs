use crate::entity::{Book, BookId, BookTitle, EventNumber};
use crate::KernelError;
use error_stack::Report;
use uuid::Uuid;

pub enum BookCommand {
    CreateBook {
        title: BookTitle,
    },
    UpdateBook {
        id: BookId,
        title: BookTitle,
        prev_number: EventNumber<Book>,
    },
    DeleteBook {
        id: BookId,
    },
}

#[async_trait::async_trait]
pub trait BookCommandHandler {
    async fn handle(&self, command: BookCommand) -> Result<Uuid, Report<KernelError>>;
}
