use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Fromln, AsRefln)]
pub struct UserRentLimit(i32);

impl UserRentLimit {
    pub fn new(limit: impl Into<i32>) -> Self {
        Self(limit.into())
    }
}
