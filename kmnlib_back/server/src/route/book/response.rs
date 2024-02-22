use axum::Json;
use crate::controller::Exhaust;
use axum::response::{IntoResponse, Response};
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, DestructBook};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct CreatedResponse {
    id: BookId,
}

impl IntoResponse for CreatedResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::CREATED, axum::Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct BookResponse {
    id: BookId,
    title: BookTitle,
    amount: BookAmount,
}

impl IntoResponse for BookResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub struct Presenter;

impl Exhaust<BookId> for Presenter {
    type To = CreatedResponse;
    fn emit(&self, input: BookId) -> Self::To {
        CreatedResponse { id: input }
    }
}

impl Exhaust<Option<Book>> for Presenter {
    type To = Option<BookResponse>;
    fn emit(&self, input: Option<Book>) -> Self::To {
        input.map(|input| {
            let DestructBook {
                id, title, amount, ..
            } = input.into_destruct();
            BookResponse { id, title, amount }
        })
    }
}

impl Exhaust<Vec<Book>> for Presenter {
    type To = Json<Vec<BookResponse>>;
    fn emit(&self, input: Vec<Book>) -> Self::To {
        let result = input
            .into_iter()
            .map(|book| {
                let DestructBook {
                    id, title, amount, ..
                } = book.into_destruct();
                BookResponse { id, title, amount }
            })
            .collect::<Vec<_>>();

        Json::from(result)
    }
}
