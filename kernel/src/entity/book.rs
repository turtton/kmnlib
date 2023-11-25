mod id;
mod title;

pub use self::{id::*, title::*};
use crate::entity::common::EventNumber;
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, References, Serialize, Deserialize, Destructure)]
pub struct Book {
    id: BookId,
    title: BookTitle,
    prev_number: EventNumber<Book>,
}

impl Book {
    pub fn new(id: BookId, title: BookTitle, prev_number: EventNumber<Book>) -> Self {
        Self {
            id,
            title,
            prev_number,
        }
    }
}
