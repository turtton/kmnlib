use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, Eq, PartialEq, Fromln, AsRefln)]
pub struct BookAmount(i32);

impl BookAmount {
    pub fn new(amount: impl Into<i32>) -> Self {
        Self(amount.into())
    }
}
