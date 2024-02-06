use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Fromln, AsRefln)]
pub struct BookTitle(String);

impl BookTitle {
    pub fn new(title: impl Into<String>) -> Self {
        Self(title.into())
    }
}
