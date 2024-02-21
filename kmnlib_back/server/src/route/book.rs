mod request;
mod response;

use crate::controller::Controller;
use crate::error::ReturnableError;
use crate::handler::AppModule;
use crate::route::book::request::{
    CreateRequest, DeleteRequest, GetRequest, Transformer, UpdateRequest,
};
use crate::route::book::response::{BookResponse, Presenter};
use application::service::{GetBookService, HandleBookService};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

pub trait BookRouter {
    fn route_book(self) -> Self;
}

impl BookRouter for Router<AppModule> {
    fn route_book(self) -> Self {
        self.route(
            "/books",
            post(
                |state: State<AppModule>, json: Json<CreateRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(json.0)
                        .handle(|event| state.0.pgpool().handle_event(event))
                        .await
                        .map_err(ReturnableError::from)
                },
            )
            .get(|state: State<AppModule>| async move { todo!() }),
        )
        .route(
            "/books/:id",
            get(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(GetRequest::new(id))
                        .handle(|dto| handler.pgpool().get_book(dto))
                        .await
                        .map_err(ReturnableError::from)
                        .map(|res| {
                            res.map(BookResponse::into_response)
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
                        .map_err(ReturnableError::from)
                },
            )
            .delete(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(DeleteRequest::new(id))
                        .handle(|command| handler.pgpool().handle_event(command))
                        .await
                        .map_err(ReturnableError::from)
                },
            ),
        )
    }
}
