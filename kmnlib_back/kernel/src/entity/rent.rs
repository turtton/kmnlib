mod returned_at;

pub use self::returned_at::*;

use crate::entity::{BookId, UserId};
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, References, Serialize, Deserialize, Destructure)]
pub struct Rent {
    book_id: BookId,
    user_id: UserId,
    returned_at: Option<ReturnedAt>,
}

impl Rent {
    pub fn new(book_id: BookId, user_id: UserId, returned_at: Option<ReturnedAt>) -> Self {
        Self {
            book_id,
            user_id,
            returned_at,
        }
    }
}
