mod amount;
mod id;
mod title;

pub use self::{amount::*, id::*, title::*};
use crate::entity::common::EventVersion;
use destructure::{Destructure, Mutation};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, References, Destructure, Mutation)]
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
