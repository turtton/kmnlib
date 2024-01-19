use std::fmt::Display;

use error_stack::Context;

#[derive(Debug)]
pub enum KernelError {
    Concurrency,
    Timeout,
    Internal,
}

impl Display for KernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelError::Concurrency => write!(f, "Concurrency error"),
            KernelError::Timeout => write!(f, "Process timed out"),
            KernelError::Internal => write!(f, "Internal kernel error"),
        }
    }
}

impl Context for KernelError {}
