use crate::controller::Exhaust;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kernel::prelude::entity::{DestructUser, User, UserId, UserName, UserRentLimit};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CreatedUserResponse {
    id: UserId,
}

impl IntoResponse for CreatedUserResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self)).into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: UserId,
    name: UserName,
    rent_limit: UserRentLimit,
}

impl IntoResponse for UserResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub struct UserPresenter;

impl Exhaust<UserId> for UserPresenter {
    type To = CreatedUserResponse;
    fn emit(&self, input: UserId) -> Self::To {
        CreatedUserResponse { id: input }
    }
}

impl Exhaust<Option<User>> for UserPresenter {
    type To = Option<UserResponse>;
    fn emit(&self, input: Option<User>) -> Self::To {
        input.map(|input| {
            let DestructUser {
                id,
                name,
                rent_limit,
                ..
            } = input.into_destruct();
            UserResponse {
                id,
                name,
                rent_limit,
            }
        })
    }
}

impl Exhaust<Vec<User>> for UserPresenter {
    type To = axum::Json<Vec<UserResponse>>;
    fn emit(&self, input: Vec<User>) -> Self::To {
        let result = input
            .into_iter()
            .map(|user| {
                let DestructUser {
                    id,
                    name,
                    rent_limit,
                    ..
                } = user.into_destruct();
                UserResponse {
                    id,
                    name,
                    rent_limit,
                }
            })
            .collect::<Vec<_>>();
        axum::Json::from(result)
    }
}
