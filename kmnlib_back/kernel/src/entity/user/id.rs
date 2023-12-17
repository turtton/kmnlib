use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Fromln, AsRefln)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self(id.into())
    }
}
