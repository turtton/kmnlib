mod id;
mod name;

pub use self::{id::*, name::*};
use crate::entity::common::EventVersion;
use destructure::Destructure;
use serde::{Deserialize, Serialize};
use vodca::References;

#[derive(Debug, Clone, Serialize, Deserialize, Destructure, References)]
pub struct User {
    id: UserId,
    name: UserName,
    version: EventVersion<User>,
}

impl User {
    pub fn new(id: UserId, name: UserName, version: EventVersion<User>) -> Self {
        Self { id, name, version }
    }
}
