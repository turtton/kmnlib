use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Fromln, AsRefln)]
pub struct EventNumber<T>(i64, PhantomData<T>);

impl<T> EventNumber<T> {
    pub fn new(id: impl Into<i64>) -> Self {
        Self(id.into(), PhantomData)
    }
}
