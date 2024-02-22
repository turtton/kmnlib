use kernel::prelude::entity::{SelectLimit, SelectOffset, UserId};

pub struct GetAllUserDto {
    pub limit: SelectLimit,
    pub offset: SelectOffset,
}

pub struct GetUserDto {
    pub id: UserId,
}
