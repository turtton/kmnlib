use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;
use vodca::{AsRefln, Fromln};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Fromln, AsRefln)]
pub struct EventVersion<T>(i64, PhantomData<T>);

impl<T> EventVersion<T> {
    pub fn new(version: i64) -> Self {
        EventVersion(version, PhantomData)
    }
}

impl<T> Serialize for EventVersion<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpectedEventVersion<T> {
    /*
     * Nothing means that there is no event stream
     */
    Nothing,
    /*
     * Exact means that there is an event stream and the version is the exact version of the event stream
     */
    Exact(EventVersion<T>),
}
