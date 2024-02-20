use error_stack::Report;
use kernel::prelude::entity::{DestructRent, Rent};
use kernel::KernelError;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RentDto {
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub version: i64,
}

impl TryFrom<Rent> for RentDto {
    type Error = Report<KernelError>;

    fn try_from(value: Rent) -> Result<Self, Self::Error> {
        let DestructRent {
            version,
            book_id,
            user_id,
        } = value.into_destruct();
        Ok(Self {
            book_id: book_id.into(),
            user_id: user_id.into(),
            version: version.try_into()?,
        })
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

pub struct CreateRentDto {
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub version: i64,
}
