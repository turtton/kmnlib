mod id;
mod name;

pub use self::{id::*, name::*};
use crate::entity::common::EventNumber;
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Serialize, Deserialize, Destructure, References)]
pub struct User {
    id: UserId,
    name: UserName,
    prev_number: EventNumber<User>,
}

impl User {
    pub fn new(id: UserId, name: UserName, prev_number: EventNumber<User>) -> Self {
        Self {
            id,
            name,
            prev_number,
        }
    }
}
