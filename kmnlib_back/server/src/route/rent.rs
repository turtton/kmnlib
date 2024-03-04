use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::request::{RentRequest, RentTransformer, ReturnRequest};
use crate::response::RentPresenter;
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
                |State(module): State<AppModule>, Query(req): Query<RentRequest>| async move {
                    Controller::new(RentTransformer, RentPresenter)
                        .intake(req)
                        .handle(|event| module.handler().pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(module): State<AppModule>, Query(req): Query<ReturnRequest>| async move {
                    Controller::new(RentTransformer, RentPresenter)
                        .intake(req)
                        .handle(|event| module.handler().pgpool().handle_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
