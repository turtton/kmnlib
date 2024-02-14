use destructure::{Destructure, Mutation};
use vodca::References;

use crate::entity::{BookId, EventVersion, UserId};

#[derive(Debug, Clone, Eq, PartialEq, References, Destructure, Mutation)]
pub struct Rent {
    version: EventVersion<Rent>,
    book_id: BookId,
    user_id: UserId,
}

impl Rent {
    pub fn new(version: EventVersion<Rent>, book_id: BookId, user_id: UserId) -> Self {
        Self {
            version,
            book_id,
            user_id,
        }
    }
}
