mod id;
mod name;
mod rent_limit;

pub use self::{id::*, name::*, rent_limit::*};
use crate::entity::common::EventVersion;
use crate::entity::IsDeleted;
use destructure::{Destructure, Mutation};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, Destructure, References, Mutation)]
pub struct User {
    id: UserId,
    name: UserName,
    rent_limit: UserRentLimit,
    version: EventVersion<User>,
    is_deleted: IsDeleted<User>,
}

impl User {
    pub fn new(
        id: UserId,
        name: UserName,
        rent_limit: UserRentLimit,
        version: EventVersion<User>,
        is_deleted: IsDeleted<User>,
    ) -> Self {
        Self {
            id,
            name,
            rent_limit,
            version,
            is_deleted,
        }
    }
}
