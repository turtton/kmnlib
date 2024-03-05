use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::request::{
    InfoLengthRequest, InfoLengthTarget, InfoRequest, InfoRequestBody, InfoTarget, InfosRequest,
    QueueTransformer,
};
use crate::response::{InfoResponse, QueuePresenter};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use kernel::interface::mq::MessageQueue;
use uuid::Uuid;

pub trait QueueRouter {
    fn route_queue(self) -> Self;
}

impl QueueRouter for Router<AppModule> {
    fn route_queue(self) -> Self {
        self.route(
            "/queue/infos",
            get(
                |State(module): State<AppModule>, Query(req): Query<InfosRequest>| async move {
                    Controller::new(QueueTransformer, QueuePresenter)
                        .intake(req)
                        .try_handle(
                            |InfosRequest {
                                 target,
                                 size,
                                 offset,
                             }| async move {
                                match target {
                                    InfoTarget::Delayed => {
                                        module
                                            .worker()
                                            .command()
                                            .get_delayed_infos(&size, &offset)
                                            .await
                                    }
                                    InfoTarget::Failed => {
                                        module
                                            .worker()
                                            .command()
                                            .get_failed_infos(&size, &offset)
                                            .await
                                    }
                                }
                            },
                        )
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/queue/infos/:id",
            get(
                |State(module): State<AppModule>,
                 Path(id): Path<Uuid>,
                 Query(req): Query<InfoRequestBody>| async move {
                    Controller::new(QueueTransformer, QueuePresenter)
                        .intake(InfoRequest::new(id, req.target))
                        .try_handle(|InfoRequest { id, target }| async move {
                            match target {
                                InfoTarget::Delayed => {
                                    module.worker().command().get_delayed_info(&id).await
                                }
                                InfoTarget::Failed => {
                                    module.worker().command().get_failed_info(&id).await
                                }
                            }
                        })
                        .await
                        .map_err(ErrorStatus::from)
                        .map(|res| {
                            res.map(InfoResponse::into_response)
                                .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
                        })
                },
            ),
        )
        .route(
            "/queue/infos/len",
            get(
                |State(module): State<AppModule>, Query(req): Query<InfoLengthRequest>| async move {
                    Controller::new(QueueTransformer, QueuePresenter)
                        .intake(req)
                        .try_handle(|InfoLengthRequest { target }| async move {
                            match target {
                                InfoLengthTarget::Queued => {
                                    module.worker().command().get_queued_len().await
                                }
                                InfoLengthTarget::Delayed => {
                                    module.worker().command().get_delayed_len().await
                                }
                                InfoLengthTarget::Failed => {
                                    module.worker().command().get_failed_len().await
                                }
                            }
                        })
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
