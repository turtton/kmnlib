use crate::entity::{BookId, UserId};
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, References, Serialize, Deserialize, Destructure)]
pub struct Rent {
    book_id: BookId,
    user_id: UserId,
}

impl Rent {
    pub fn new(book_id: BookId, user_id: UserId) -> Self {
        Self { book_id, user_id }
    }
}
