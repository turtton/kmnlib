mod book;
mod rent;
mod user;

pub use self::{book::*, rent::*, user::*};
use crate::entity::EventVersion;
use destructure::Destructure;
use vodca::References;

#[derive(Debug, Clone, Eq, PartialEq, References, Destructure)]
pub struct EventInfo<Event, Entity> {
    event: Event,
    version: EventVersion<Entity>,
}

impl<Event, Entity> EventInfo<Event, Entity> {
    pub fn new(event: Event, version: EventVersion<Entity>) -> Self {
        Self { event, version }
    }
}
