use serde::{Deserialize, Serialize};
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Fromln, AsRefln, Serialize, Deserialize)]
pub struct UserRentLimit(i32);

impl UserRentLimit {
    pub fn new(limit: impl Into<i32>) -> Self {
        Self(limit.into())
    }
}
