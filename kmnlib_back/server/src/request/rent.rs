use crate::controller::Intake;
use crate::request::{BookTransformer, UserTransformer};
use application::transfer::{GetRentFromBookIdDto, GetRentFromUserIdDto};
use kernel::interface::event::RentEvent;
use kernel::prelude::entity::{BookId, UserId};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RentRequest {
    book_id: Uuid,
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ReturnRequest {
    book_id: Uuid,
    user_id: Uuid,
}

#[derive(Debug)]
pub struct GetRentsRequest {
    id: Uuid,
}

impl GetRentsRequest {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl Intake<GetRentsRequest> for BookTransformer {
    type To = GetRentFromBookIdDto;
    fn emit(&self, input: GetRentsRequest) -> Self::To {
        GetRentFromBookIdDto {
            book_id: BookId::new(input.id),
        }
    }
}

impl Intake<GetRentsRequest> for UserTransformer {
    type To = GetRentFromUserIdDto;
    fn emit(&self, input: GetRentsRequest) -> Self::To {
        GetRentFromUserIdDto {
            user_id: UserId::new(input.id),
        }
    }
}

pub struct RentTransformer;

impl Intake<RentRequest> for RentTransformer {
    type To = RentEvent;
    fn emit(&self, RentRequest { book_id, user_id }: RentRequest) -> Self::To {
        Self::To::Rent {
            book_id: BookId::new(book_id),
            user_id: UserId::new(user_id),
        }
    }
}

impl Intake<ReturnRequest> for RentTransformer {
    type To = RentEvent;
    fn emit(&self, ReturnRequest { book_id, user_id }: ReturnRequest) -> Self::To {
        Self::To::Return {
            book_id: BookId::new(book_id),
            user_id: UserId::new(user_id),
        }
    }
}
