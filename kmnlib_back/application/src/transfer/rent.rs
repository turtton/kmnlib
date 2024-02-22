use kernel::prelude::entity::{BookId, UserId};

pub struct GetRentFromBookIdDto {
    pub book_id: BookId,
}

pub struct GetRentFromUserIdDto {
    pub user_id: UserId,
}

pub struct GetRentFromIdDto {
    pub book_id: BookId,
    pub user_id: UserId,
}
