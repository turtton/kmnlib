use crate::error::StackTrace;
use crate::handler::AppModule;
use crate::route::{BookRouter, RentRouter, UserRouter};
use error_stack::ResultExt;
use kernel::KernelError;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

mod controller;
mod error;
mod handler;
mod mq;
mod request;
mod response;
mod route;

#[tokio::main]
async fn main() -> Result<(), StackTrace> {
    let appender = tracing_appender::rolling::daily(std::path::Path::new("./logs/"), "debug.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::EnvFilter::new(
                    std::env::var("RUST_LOG").unwrap_or_else(|_| {
                        "driver=debug,server=debug,tower_http=debug,hyper=debug,sqlx=debug".into()
                    }),
                ))
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        )
        .with(
            tracing_subscriber::fmt::Layer::default()
                .with_writer(non_blocking_appender)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        )
        .init();

    let app = AppModule::new().await?;

    let router = axum::Router::new()
        .route_book()
        .route_user()
        .route_rent()
        .layer(
            CorsLayer::new(), //TODO .allow_origin([""])
        )
        .with_state(app);

    let bind = SocketAddr::from(([0, 0, 0, 0], 8080));
    let tcp = TcpListener::bind(bind)
        .await
        .change_context_lazy(|| KernelError::Internal)
        .attach_printable_lazy(|| "Failed to listen tcp")?;

    axum::serve(tcp, router.into_make_service())
        .await
        .change_context_lazy(|| KernelError::Internal)?;

    Ok(())
}
