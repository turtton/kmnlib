use crate::controller::Intake;
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

pub struct Transformer;

impl Intake<RentRequest> for Transformer {
    type To = RentEvent;
    fn emit(&self, RentRequest { book_id, user_id }: RentRequest) -> Self::To {
        Self::To::Rent {
            book_id: BookId::new(book_id),
            user_id: UserId::new(user_id),
        }
    }
}

impl Intake<ReturnRequest> for Transformer {
    type To = RentEvent;
    fn emit(&self, ReturnRequest { book_id, user_id }: ReturnRequest) -> Self::To {
        Self::To::Return {
            book_id: BookId::new(book_id),
            user_id: UserId::new(user_id),
        }
    }
}
