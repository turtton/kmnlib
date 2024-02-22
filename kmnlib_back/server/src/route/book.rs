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
use uuid::Uuid;

pub trait BookRouter {
    fn route_book(self) -> Self;
}

impl BookRouter for Router<AppModule> {
    fn route_book(self) -> Self {
        self.route(
            "/books",
            get(
                |State(handler): State<AppModule>, Query(req): Query<GetAllBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(req)
                        .handle(|dto| async move { handler.pgpool().get_all(&dto).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .post(
                |State(handler): State<AppModule>, Json(req): Json<CreateBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(req)
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/books/:id",
            get(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(GetBookRequest::new(id))
                        .handle(|dto| async move { handler.pgpool().get_book(&dto).await })
                        .await
                        .map_err(ErrorStatus::from)
                        .map(|res| {
                            res.map(BookResponse::into_response)
                                .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
                        })
                },
            )
            .patch(
                |State(handler): State<AppModule>,
                 Path(id): Path<Uuid>,
                 Json(req): Json<UpdateBookRequest>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake((id, req))
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, BookPresenter)
                        .intake(DeleteBookRequest::new(id))
                        .handle(|command| handler.pgpool().handle_event(command))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/books/:id/rents",
            get(
                |State(handler): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(BookTransformer, RentPresenter)
                        .intake(GetRentsRequest::new(id))
                        .handle(
                            |dto| async move { handler.pgpool().get_rent_from_book(&dto).await },
                        )
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
