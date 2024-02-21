use crate::KernelError;
use error_stack::{Report, ResultExt};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl<T> TryFrom<EventVersion<T>> for i64 {
    type Error = Report<KernelError>;
    fn try_from(value: EventVersion<T>) -> Result<Self, Self::Error> {
        match value {
            EventVersion::Nothing => Err(Report::new(KernelError::Internal))
                .attach_printable_lazy(|| "Cannot convert EventVersion::Nothing to i64"),
            EventVersion::Exact(version, _) => Ok(version),
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

impl<T> Serialize for EventVersion<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Nothing => (-1).serialize(serializer),
            Self::Exact(version, _) => version.serialize(serializer),
        }
    }
}

impl<'de, T> Deserialize<'de> for EventVersion<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <i64>::deserialize(deserializer).map(|version| Self::new(version))
    }
}
