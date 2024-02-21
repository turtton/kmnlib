use destructure::Destructure;
use error_stack::Report;
use vodca::References;

use crate::entity::{CreatedAt, EventVersion, ExpectedEventVersion};
use crate::KernelError;

pub use self::{book::*, rent::*, user::*};

mod book;
mod rent;
mod user;

#[derive(Debug, Clone, Eq, PartialEq, References, Destructure)]
pub struct EventInfo<Event, Entity> {
    event: Event,
    version: EventVersion<Entity>,
    created_at: CreatedAt<Entity>,
}

impl<Event, Entity> EventInfo<Event, Entity> {
    pub fn new(event: Event, version: EventVersion<Entity>, created_at: CreatedAt<Entity>) -> Self {
        Self {
            event,
            version,
            created_at,
        }
    }
}

pub trait Applier<Event>: 'static + Sync + Send {
    fn apply(&mut self, event: Event);
}

#[derive(Debug, Clone, Eq, PartialEq, Destructure)]
pub struct CommandInfo<Event, Entity> {
    event: Event,
    version: Option<ExpectedEventVersion<Entity>>,
}

impl<Event, Entity> CommandInfo<Event, Entity> {
    pub fn new(event: Event, version: Option<ExpectedEventVersion<Entity>>) -> Self {
        Self { event, version }
    }
}

pub(in crate::event) trait EventRowFieldAttachments {
    fn attach_field_details(self, event_name: &str, field_name: &str) -> Self;
    fn attach_unknown_event(self, entity_name: &str, event_name: &str) -> Self;
}

impl EventRowFieldAttachments for Report<KernelError> {
    fn attach_field_details(self, event_name: &str, field_name: &str) -> Self {
        self.attach_printable(format!(
            "Failed to get raw field. Event: {event_name}, Field: {field_name}"
        ))
    }

    fn attach_unknown_event(self, entity_name: &str, event_name: &str) -> Self {
        self.attach_printable(format!("Unknown {entity_name} event name: {event_name}"))
    }
}
