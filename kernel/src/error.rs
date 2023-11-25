use error_stack::Context;
use std::fmt::Display;

#[derive(Debug)]
pub struct KernelError;

impl Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to execute driver function")
    }
}

impl Context for KernelError {}
