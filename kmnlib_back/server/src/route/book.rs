use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::request::{
    BookTransformer, CreateBookRequest, DeleteBookRequest, GetAllBookRequest, GetBookRequest,
    GetRentsRequest, UpdateBookRequest,
};
use crate::response::{BookPresenter, BookResponse, RentPresenter};
use application::service::{GetBookService, GetRentService, HandleBookService};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use kernel::interface::mq::MessageQueue;
use uuid::Uuid;

pub trait BookRouter {
    fn route_book(self) -> Self;
}

impl BookRouter for Router<AppModule> {
    fn route_book(self) -> Self {
        self.route(
            "/books",
            get(
                |State(module): State<AppModule>, Query(req): Query<GetAllBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(req)
                        .handle(|dto| async move { module.handler().pgpool().get_all(&dto).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .post(
                |State(module): State<AppModule>, Json(req): Json<CreateBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(req)
                        .handle(|event| module.handler().pgpool().handle_book_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/books/:id",
            get(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(GetBookRequest::new(id))
                        .handle(|dto| async move { module.handler().pgpool().get_book(&dto).await })
                        .await
                        .map_err(ErrorStatus::from)
                        .map(|res| {
                            res.map(BookResponse::into_response)
                                .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
                        })
                },
            )
            .patch(
                |State(module): State<AppModule>,
                 Path(id): Path<Uuid>,
                 Json(req): Json<UpdateBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake((id, req))
                        .handle(|info| async move { module.worker().command().queue(&info).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(DeleteBookRequest::new(id))
                        .handle(|info| async move { module.worker().command().queue(&info).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/books/:id/rents",
            get(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, RentPresenter)
                        .intake(GetRentsRequest::new(id))
                        .handle(|dto| async move {
                            module.handler().pgpool().get_rent_from_book(&dto).await
                        })
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
