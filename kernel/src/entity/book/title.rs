use serde::{Deserialize, Serialize};
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Fromln, AsRefln)]
pub struct BookTitle(String);

impl BookTitle {
    pub fn new(title: impl Into<String>) -> Self {
        Self(title.into())
    }
}
