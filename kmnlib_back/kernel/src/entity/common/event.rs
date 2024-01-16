use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventVersion<T> {
    /*
     * Nothing means that there is no event stream
     */
    Nothing,
    /*
     * Exact means that there is an event stream and the version is the exact version of the event stream
     */
    Exact(i64, PhantomData<T>),
}

impl<T> EventVersion<T> {
    pub fn new(version: i64) -> Self {
        if version < 0 {
            Self::Nothing
        } else {
            Self::Exact(version, PhantomData)
        }
    }
}

impl<T> From<i64> for EventVersion<T> {
    fn from(version: i64) -> Self {
        if version < 0 {
            Self::Nothing
        } else {
            Self::Exact(version, PhantomData)
        }
    }
}

impl<T> AsRef<i64> for EventVersion<T> {
    fn as_ref(&self) -> &i64 {
        match self {
            Self::Nothing => &-1,
            Self::Exact(version, _) => version,
        }
    }
}
