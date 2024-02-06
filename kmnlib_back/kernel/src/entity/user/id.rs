use uuid::Uuid;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default, Fromln, AsRefln)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self(id.into())
    }
}
