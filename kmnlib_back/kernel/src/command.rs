use crate::entity::EventVersion;
use destructure::Destructure;

#[derive(Debug, Clone, Eq, PartialEq, Destructure)]
pub struct CommandInfo<Event, Entity> {
    event: Event,
    version: Option<EventVersion<Entity>>,
}

impl<Event, Entity> CommandInfo<Event, Entity> {
    pub fn new(event: Event, version: Option<EventVersion<Entity>>) -> Self {
        Self { event, version }
    }
}
