use crate::controller::Intake;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub enum InfoTarget {
    #[serde(rename = "delayed")]
    Delayed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct InfosRequest {
    pub target: InfoTarget,
    pub size: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct InfoRequestBody {
    pub target: InfoTarget,
}

#[derive(Debug, Deserialize)]
pub enum InfoLengthTarget {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "delayed")]
    Delayed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct InfoLengthRequest {
    pub target: InfoLengthTarget,
}

#[derive(Debug)]
pub struct InfoRequest {
    pub id: Uuid,
    pub target: InfoTarget,
}

impl InfoRequest {
    pub fn new(id: Uuid, target: InfoTarget) -> Self {
        Self { id, target }
    }
}

pub struct QueueTransformer;

impl<T> Intake<T> for QueueTransformer {
    type To = T;
    fn emit(&self, input: T) -> Self::To {
        input
    }
}
