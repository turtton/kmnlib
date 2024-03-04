use crate::controller::Intake;
use crate::mq::CommandOperation;
use application::transfer::{GetAllBookDto, GetBookDto};
use kernel::interface::event::BookEvent;
use kernel::interface::mq::QueueInfo;
use kernel::prelude::entity::{BookAmount, BookId, BookTitle, SelectLimit, SelectOffset};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    title: String,
    amount: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    title: Option<String>,
    amount: Option<i32>,
}

#[derive(Debug)]
pub struct DeleteBookRequest {
    id: Uuid,
}

impl DeleteBookRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

// I want to use primitive type(i32) in these fields, but default attribute not supported for literals(https://github.com/serde-rs/serde/issues/368)
#[derive(Debug, Deserialize)]
pub struct GetAllBookRequest {
    #[serde(default)]
    limit: SelectLimit,
    #[serde(default)]
    offset: SelectOffset,
}

#[derive(Debug)]
pub struct GetBookRequest {
    id: Uuid,
}

impl GetBookRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

pub struct BookTransformer;

impl Intake<CreateBookRequest> for BookTransformer {
    type To = BookEvent;
    fn emit(&self, input: CreateBookRequest) -> Self::To {
        Self::To::Create {
            id: BookId::new(Uuid::new_v4()),
            title: BookTitle::new(input.title),
            amount: BookAmount::new(input.amount),
        }
    }
}

impl Intake<(Uuid, UpdateBookRequest)> for BookTransformer {
    type To = QueueInfo<CommandOperation>;
    fn emit(&self, input: (Uuid, UpdateBookRequest)) -> Self::To {
        let (id, input) = input;
        let operation = CommandOperation::book(BookEvent::Update {
            id: BookId::new(id),
            title: input.title.map(BookTitle::new),
            amount: input.amount.map(BookAmount::new),
        });
        Self::To::from(operation)
    }
}

impl Intake<DeleteBookRequest> for BookTransformer {
    type To = QueueInfo<CommandOperation>;
    fn emit(&self, input: DeleteBookRequest) -> Self::To {
        let operation = CommandOperation::book(BookEvent::Delete {
            id: BookId::new(input.id),
        });
        Self::To::from(operation)
    }
}

impl Intake<GetBookRequest> for BookTransformer {
    type To = GetBookDto;
    fn emit(&self, input: GetBookRequest) -> Self::To {
        GetBookDto {
            id: BookId::new(input.id),
        }
    }
}

impl Intake<GetAllBookRequest> for BookTransformer {
    type To = GetAllBookDto;
    fn emit(&self, input: GetAllBookRequest) -> Self::To {
        GetAllBookDto {
            limit: input.limit,
            offset: input.offset,
        }
    }
}
