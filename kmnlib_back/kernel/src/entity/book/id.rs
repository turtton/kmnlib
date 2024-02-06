use uuid::Uuid;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Fromln, AsRefln)]
pub struct BookId(Uuid);

impl BookId {
    pub fn new(id: impl Into<Uuid>) -> Self {
        Self(id.into())
    }
}
