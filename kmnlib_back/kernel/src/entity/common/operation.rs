use serde::{Deserialize, Serialize};
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Fromln, AsRefln, Serialize, Deserialize)]
pub struct SelectLimit(i32);

impl SelectLimit {
    pub fn new(value: impl Into<i32>) -> Self {
        SelectLimit(value.into())
    }
}

impl Default for SelectLimit {
    fn default() -> Self {
        Self::new(30)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Fromln, AsRefln, Serialize, Deserialize)]
pub struct SelectOffset(i32);

impl SelectOffset {
    pub fn new(value: impl Into<i32>) -> Self {
        SelectOffset(value.into())
    }
}
