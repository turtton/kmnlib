use crate::controller::Exhaust;
use axum::response::{IntoResponse, Response};
use kernel::prelude::entity::{Book, BookAmount, BookId, BookTitle, DestructBook};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CreatedBookResponse {
    id: BookId,
}

impl IntoResponse for CreatedBookResponse {
    fn into_response(self) -> Response {
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
    fn into_response(self) -> Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub struct BookPresenter;

impl Exhaust<()> for BookPresenter {
    type To = ();
    fn emit(&self, input: ()) -> Self::To {
        input
    }
}

impl Exhaust<BookId> for BookPresenter {
    type To = CreatedBookResponse;
    fn emit(&self, input: BookId) -> Self::To {
        CreatedBookResponse { id: input }
    }
}

impl Exhaust<Option<Book>> for BookPresenter {
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

impl Exhaust<Vec<Book>> for BookPresenter {
    type To = axum::Json<Vec<BookResponse>>;
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

        axum::Json::from(result)
    }
}
