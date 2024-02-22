use kernel::prelude::entity::{BookId, SelectLimit, SelectOffset};

pub struct GetAllBookDto {
    pub limit: SelectLimit,
    pub offset: SelectOffset,
}

pub struct GetBookDto {
    pub id: BookId,
}
