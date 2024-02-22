mod request;
mod response;

use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::route::rent::request::{RentRequest, ReturnRequest, Transformer};
use crate::route::rent::response::Presenter;
use application::service::HandleRentService;
use axum::extract::{Query, State};
use axum::routing::post;
use axum::Router;

pub trait RentRouter {
    fn route_rent(self) -> Self;
}

impl RentRouter for Router<AppModule> {
    fn route_rent(self) -> Self {
        self.route(
            "/rents",
            post(
                |State(handler): State<AppModule>, Query(req): Query<RentRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(req)
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(handler): State<AppModule>, Query(req): Query<ReturnRequest>| async move {
                    Controller::new(Transformer, Presenter)
                        .intake(req)
                        .handle(|event| handler.pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
