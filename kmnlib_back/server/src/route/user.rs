use crate::controller::Controller;
use crate::error::ErrorStatus;
use crate::handler::AppModule;
use crate::request::{
    CreateUserRequest, DeleteUserRequest, GetAllUserRequest, GetRentsRequest, GetUserRequest,
    UpdateUserRequest, UserTransformer,
};
use crate::response::{RentPresenter, UserPresenter, UserResponse};
use application::service::{GetRentService, GetUserService, HandleUserService};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use kernel::interface::mq::MessageQueue;
use uuid::Uuid;

pub trait UserRouter {
    fn route_user(self) -> Self;
}

impl UserRouter for Router<AppModule> {
    fn route_user(self) -> Self {
        self.route(
            "/users",
            get(
                |State(module): State<AppModule>, Query(req): Query<GetAllUserRequest>| async move {
                    Controller::new(UserTransformer, UserPresenter)
                        .intake(req)
                        .handle(|dto| module.handler().pgpool().get_all(dto))
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .post(
                |State(module): State<AppModule>, Json(req): Json<CreateUserRequest>| async move {
                    Controller::new(UserTransformer, UserPresenter)
                        .intake(req)
                        .handle(|event| module.handler().pgpool().handle_user_event(event))
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/users/:id",
            get(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(UserTransformer, UserPresenter)
                        .intake(GetUserRequest::new(id))
                        .handle(|dto| async move { module.handler().pgpool().get_user(&dto).await })
                        .await
                        .map_err(ErrorStatus::from)
                        .map(|res| {
                            res.map(UserResponse::into_response)
                                .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
                        })
                },
            )
            .patch(
                |State(module): State<AppModule>,
                 Path(id): Path<Uuid>,
                 Json(req): Json<UpdateUserRequest>| async move {
                    Controller::new(UserTransformer, UserPresenter)
                        .intake((id, req))
                        .handle(|info| async move { module.worker().command().queue(&info).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            )
            .delete(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(UserTransformer, UserPresenter)
                        .intake(DeleteUserRequest::new(id))
                        .handle(|info| async move { module.worker().command().queue(&info).await })
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
        .route(
            "/users/:id/rents",
            get(
                |State(module): State<AppModule>, Path(id): Path<Uuid>| async move {
                    Controller::new(UserTransformer, RentPresenter)
                        .intake(GetRentsRequest::new(id))
                        .handle(|dto| async move {
                            module.handler().pgpool().get_rents_from_user(&dto).await
                        })
                        .await
                        .map_err(ErrorStatus::from)
                },
            ),
        )
    }
}
