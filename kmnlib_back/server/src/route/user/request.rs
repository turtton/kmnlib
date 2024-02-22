use crate::controller::Intake;
use application::transfer::{GetAllUserDto, GetUserDto};
use kernel::interface::event::UserEvent;
use kernel::prelude::entity::{SelectLimit, SelectOffset, UserId, UserName, UserRentLimit};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    name: String,
    rent_limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    name: Option<String>,
    rent_limit: Option<i32>,
}

#[derive(Debug)]
pub struct DeleteRequest {
    id: Uuid,
}

impl DeleteRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetAllRequest {
    #[serde(default)]
    limit: SelectLimit,
    #[serde(default)]
    offset: SelectOffset,
}

#[derive(Debug)]
pub struct GetRequest {
    id: Uuid,
}

impl GetRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

pub struct Transformer;

impl Intake<CreateRequest> for Transformer {
    type To = UserEvent;
    fn emit(&self, input: CreateRequest) -> Self::To {
        Self::To::Create {
            id: UserId::new(Uuid::new_v4()),
            name: UserName::new(input.name),
            rent_limit: UserRentLimit::new(input.rent_limit),
        }
    }
}

impl Intake<(Uuid, UpdateRequest)> for Transformer {
    type To = UserEvent;
    fn emit(&self, (id, req): (Uuid, UpdateRequest)) -> Self::To {
        Self::To::Update {
            id: UserId::new(id),
            name: req.name.map(UserName::new),
            rent_limit: req.rent_limit.map(UserRentLimit::new),
        }
    }
}

impl Intake<DeleteRequest> for Transformer {
    type To = UserEvent;
    fn emit(&self, input: DeleteRequest) -> Self::To {
        Self::To::Delete {
            id: UserId::new(input.id),
        }
    }
}

impl Intake<GetRequest> for Transformer {
    type To = GetUserDto;
    fn emit(&self, input: GetRequest) -> Self::To {
        GetUserDto {
            id: UserId::new(input.id),
        }
    }
}

impl Intake<GetAllRequest> for Transformer {
    type To = GetAllUserDto;
    fn emit(&self, input: GetAllRequest) -> Self::To {
        GetAllUserDto {
            limit: input.limit,
            offset: input.offset,
        }
    }
}
