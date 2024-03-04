use crate::controller::Intake;
use crate::mq::CommandOperation;
use application::transfer::{GetAllUserDto, GetUserDto};
use kernel::interface::event::UserEvent;
use kernel::interface::mq::QueueInfo;
use kernel::prelude::entity::{SelectLimit, SelectOffset, UserId, UserName, UserRentLimit};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    name: String,
    rent_limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    name: Option<String>,
    rent_limit: Option<i32>,
}

#[derive(Debug)]
pub struct DeleteUserRequest {
    id: Uuid,
}

impl DeleteUserRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetAllUserRequest {
    #[serde(default)]
    limit: SelectLimit,
    #[serde(default)]
    offset: SelectOffset,
}

#[derive(Debug)]
pub struct GetUserRequest {
    id: Uuid,
}

impl GetUserRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

pub struct UserTransformer;

impl Intake<CreateUserRequest> for UserTransformer {
    type To = UserEvent;
    fn emit(&self, input: CreateUserRequest) -> Self::To {
        Self::To::Create {
            id: UserId::new(Uuid::new_v4()),
            name: UserName::new(input.name),
            rent_limit: UserRentLimit::new(input.rent_limit),
        }
    }
}

impl Intake<(Uuid, UpdateUserRequest)> for UserTransformer {
    type To = QueueInfo<CommandOperation>;
    fn emit(&self, (id, req): (Uuid, UpdateUserRequest)) -> Self::To {
        let operation = CommandOperation::user(UserEvent::Update {
            id: UserId::new(id),
            name: req.name.map(UserName::new),
            rent_limit: req.rent_limit.map(UserRentLimit::new),
        });
        Self::To::from(operation)
    }
}

impl Intake<DeleteUserRequest> for UserTransformer {
    type To = QueueInfo<CommandOperation>;
    fn emit(&self, input: DeleteUserRequest) -> Self::To {
        let operation = CommandOperation::user(UserEvent::Delete {
            id: UserId::new(input.id),
        });
        Self::To::from(operation)
    }
}

impl Intake<GetUserRequest> for UserTransformer {
    type To = GetUserDto;
    fn emit(&self, input: GetUserRequest) -> Self::To {
        GetUserDto {
            id: UserId::new(input.id),
        }
    }
}

impl Intake<GetAllUserRequest> for UserTransformer {
    type To = GetAllUserDto;
    fn emit(&self, input: GetAllUserRequest) -> Self::To {
        GetAllUserDto {
            limit: input.limit,
            offset: input.offset,
        }
    }
}
