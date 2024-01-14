mod id;
mod name;
mod rent_limit;

pub use self::{id::*, name::*, rent_limit::*};
use crate::entity::common::EventVersion;
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Destructure, References)]
pub struct User {
    id: UserId,
    name: UserName,
    rent_limit: UserRentLimit,
    version: EventVersion<User>,
}

impl User {
    pub fn new(
        id: UserId,
        name: UserName,
        rent_limit: UserRentLimit,
        version: EventVersion<User>,
    ) -> Self {
        Self {
            id,
            name,
            rent_limit,
            version,
        }
    }
}
