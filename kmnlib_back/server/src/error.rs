use axum::http::StatusCode;
use axum::response::IntoResponse;
use error_stack::Report;
use kernel::KernelError;
use std::process::{ExitCode, Termination};

#[derive(Debug)]
pub struct StackTrace(Report<KernelError>);

impl From<Report<KernelError>> for StackTrace {
    fn from(e: Report<KernelError>) -> Self {
        StackTrace(e)
    }
}

impl Termination for StackTrace {
    fn report(self) -> ExitCode {
        self.0.report()
    }
}

#[derive(Debug)]
pub struct ReturnableError(Report<KernelError>);

impl From<Report<KernelError>> for ReturnableError {
    fn from(e: Report<KernelError>) -> Self {
        ReturnableError(e)
    }
}

impl IntoResponse for ReturnableError {
    fn into_response(self) -> axum::response::Response {
        match self.0.current_context() {
            KernelError::Concurrency => StatusCode::CONFLICT,
            KernelError::Timeout => StatusCode::REQUEST_TIMEOUT,
            KernelError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}
