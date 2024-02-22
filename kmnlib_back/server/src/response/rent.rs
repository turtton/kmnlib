use crate::controller::Exhaust;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kernel::prelude::entity::{BookId, DestructRent, Rent, ReturnedAt, UserId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RentResponse {
    book_id: BookId,
    user_id: UserId,
    returned_at: Option<ReturnedAt>,
}

impl IntoResponse for RentResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub struct RentPresenter;

impl Exhaust<()> for RentPresenter {
    type To = ();
    fn emit(&self, input: ()) -> Self::To {
        input
    }
}

impl Exhaust<Vec<Rent>> for RentPresenter {
    type To = axum::Json<Vec<RentResponse>>;
    fn emit(&self, input: Vec<Rent>) -> Self::To {
        let result = input
            .into_iter()
            .map(|rent| {
                let DestructRent {
                    book_id,
                    user_id,
                    returned_at,
                    ..
                } = rent.into_destruct();
                RentResponse {
                    book_id,
                    user_id,
                    returned_at: returned_at.map(|tuple| tuple.0),
                }
            })
            .collect::<Vec<_>>();
        axum::Json::from(result)
    }
}
