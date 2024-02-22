use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::route::user::request::{
    CreateRequest, DeleteRequest, GetAllRequest, GetRequest, Transformer, UpdateRequest,
};
use crate::route::user::response::{Presenter, UserResponse};
use application::service::{GetUserService, HandleUserService};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

mod request;
mod response;

pub trait UserRouter {
    fn route_user(self) -> Self;
}

impl UserRouter for Router<AppModule> {
    fn route_user(self) -> Self {
        self.route(
            "/users",
            get(
                |State(handler): State<AppModule>, Query(req): Query<GetAllRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(req)
                        .handle(|dto| handler.pgpool().get_all(dto))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .post(
                |State(handler): State<AppModule>, Json(req): Json<CreateRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(req)
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/users/:id",
            get(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(GetRequest::new(id))
                        .handle(|dto| handler.pgpool().get_user(dto))
                        .await
                        .map_err(ErrorStatus::from)
                        .map(|res| {
                            res.map(UserResponse::into_response)
                                .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
                        })
                },
            )
            .patch(
                |State(handler): State<AppModule>,
                 Path(id): Path<Uuid>,
                 Json(req): Json<UpdateRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake((id, req))
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(DeleteRequest::new(id))
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
