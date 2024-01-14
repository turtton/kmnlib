mod amount;
mod id;
mod title;

pub use self::{amount::*, id::*, title::*};
use crate::entity::common::EventVersion;
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, References, Serialize, Deserialize, Destructure)]
pub struct Book {
    id: BookId,
    title: BookTitle,
    amount: BookAmount,
    version: EventVersion<Book>,
}

impl Book {
    pub fn new(
        id: BookId,
        title: BookTitle,
        amount: BookAmount,
        version: EventVersion<Book>,
    ) -> Self {
        Self {
            id,
            title,
            amount,
            version,
        }
    }
}
