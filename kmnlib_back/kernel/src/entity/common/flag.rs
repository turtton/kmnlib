use std::marker::PhantomData;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Fromln, AsRefln)]
pub struct IsDeleted<T>(bool, PhantomData<T>);

impl<T> IsDeleted<T> {
    pub fn new(value: impl Into<bool>) -> Self {
        IsDeleted(value.into(), PhantomData)
    }
}
