mod returned_at;

pub use crate::entity::rent::returned_at::*;

use destructure::{Destructure, Mutation};
use vodca::References;

use crate::entity::{BookId, EventVersion, UserId};

#[derive(Debug, Clone, Eq, PartialEq, References, Destructure, Mutation)]
pub struct Rent {
    version: EventVersion<Rent>,
    book_id: BookId,
    user_id: UserId,
    returned_at: Option<(ReturnedAt, EventVersion<Rent>)>,
}

impl Rent {
    pub fn new(
        version: EventVersion<Rent>,
        book_id: BookId,
        user_id: UserId,
        returned_at: Option<(ReturnedAt, EventVersion<Rent>)>,
    ) -> Self {
        Self {
            version,
            book_id,
            user_id,
            returned_at,
        }
    }
}
