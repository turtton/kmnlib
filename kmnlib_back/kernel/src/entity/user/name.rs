use serde::{Deserialize, Serialize};
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Fromln, AsRefln)]
pub struct UserName(String);

impl UserName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
