use crate::controller::Intake;
use application::transfer::GetBookDto;
use kernel::interface::event::BookEvent;
use kernel::prelude::entity::{BookAmount, BookId, BookTitle};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    title: String,
    amount: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRequest {
    title: Option<String>,
    amount: Option<i32>,
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
    type To = BookEvent;
    fn emit(&self, input: CreateRequest) -> Self::To {
        Self::To::Create {
            id: BookId::new(Uuid::new_v4()),
            title: BookTitle::new(input.title),
            amount: BookAmount::new(input.amount),
        }
    }
}

impl Intake<(Uuid, UpdateRequest)> for Transformer {
    type To = BookEvent;
    fn emit(&self, input: (Uuid, UpdateRequest)) -> Self::To {
        let (id, input) = input;
        Self::To::Update {
            id: BookId::new(id),
            title: input.title.map(BookTitle::new),
            amount: input.amount.map(BookAmount::new),
        }
    }
}

impl Intake<DeleteRequest> for Transformer {
    type To = BookEvent;
    fn emit(&self, input: DeleteRequest) -> Self::To {
        Self::To::Delete {
            id: BookId::new(input.id),
        }
    }
}

impl Intake<GetRequest> for Transformer {
    type To = GetBookDto;
    fn emit(&self, input: GetRequest) -> Self::To {
        GetBookDto {
            id: BookId::new(input.id),
        }
    }
}
