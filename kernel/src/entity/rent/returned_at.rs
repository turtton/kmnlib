use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, AsRefln, Fromln)]
pub struct ReturnedAt(OffsetDateTime);

impl ReturnedAt {
    pub fn new(time: impl Into<OffsetDateTime>) -> Self {
        Self(time.into())
    }
}
