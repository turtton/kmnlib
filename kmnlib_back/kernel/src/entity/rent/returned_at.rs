use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Fromln, AsRefln, Serialize, Deserialize)]
pub struct ReturnedAt(OffsetDateTime);

impl ReturnedAt {
    pub fn new(time: impl Into<OffsetDateTime>) -> Self {
        Self(time.into())
    }
}
