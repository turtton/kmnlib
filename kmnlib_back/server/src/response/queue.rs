use crate::controller::TryExhaust;
use crate::mq::CommandOperation;
use axum::response::IntoResponse;
use error_stack::{Report, ResultExt};
use kernel::interface::mq::{DestructErroredInfo, ErroredInfo};
use kernel::KernelError;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub id: Uuid,
    pub data: String,
    pub stack_trace: String,
}

impl IntoResponse for InfoResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct InfoLengthResponse {
    pub length: u64,
}

impl IntoResponse for InfoLengthResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub struct QueuePresenter;

impl TryExhaust<ErroredInfo<CommandOperation>> for QueuePresenter {
    type To = InfoResponse;

    type Error = Report<KernelError>;
    fn emit(&self, input: ErroredInfo<CommandOperation>) -> Result<Self::To, Self::Error> {
        let DestructErroredInfo {
            id,
            data,
            stack_trace,
        } = input.into_destruct();
        let data = serde_json::to_string(&data).change_context_lazy(|| KernelError::Internal)?;
        Ok(InfoResponse {
            id,
            data,
            stack_trace,
        })
    }
}

impl TryExhaust<Vec<ErroredInfo<CommandOperation>>> for QueuePresenter {
    type To = axum::Json<Vec<InfoResponse>>;
    type Error = Report<KernelError>;
    fn emit(&self, input: Vec<ErroredInfo<CommandOperation>>) -> Result<Self::To, Self::Error> {
        let infos = input
            .into_iter()
            .map(|info| self.emit(info))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(axum::Json(infos))
    }
}

impl TryExhaust<Option<ErroredInfo<CommandOperation>>> for QueuePresenter {
    type To = Option<InfoResponse>;
    type Error = Report<KernelError>;
    fn emit(&self, input: Option<ErroredInfo<CommandOperation>>) -> Result<Self::To, Self::Error> {
        input.map(|info| self.emit(info)).transpose()
    }
}

impl TryExhaust<usize> for QueuePresenter {
    type To = InfoLengthResponse;
    type Error = Report<KernelError>;
    fn emit(&self, input: usize) -> Result<Self::To, Self::Error> {
        let length = u64::try_from(input).change_context_lazy(|| KernelError::Internal)?;
        Ok(InfoLengthResponse { length })
    }
}
