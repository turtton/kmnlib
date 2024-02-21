use kernel::prelude::entity::{DestructRent, Rent};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RentDto {
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub version: i64,
}

impl From<Rent> for RentDto {
    fn from(value: Rent) -> Self {
        let DestructRent {
            version,
            book_id,
            user_id,
        } = value.into_destruct();
        Self {
            book_id: book_id.into(),
            user_id: user_id.into(),
            version: version.into(),
        }
    }
}

pub struct GetRentFromBookIdDto {
    pub book_id: Uuid,
}

pub struct GetRentFromUserIdDto {
    pub user_id: Uuid,
}

pub struct GetRentFromIdDto {
    pub book_id: Uuid,
    pub user_id: Uuid,
}
