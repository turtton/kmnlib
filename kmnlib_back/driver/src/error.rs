use error_stack::Context;
use std::fmt::Display;

#[derive(Debug)]
pub enum DriverError {
    SqlX,
    EventStore,
    Env,
    Conversion,
    Serde,
}

impl Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to execute driver function")
    }
}

impl Context for DriverError {}
