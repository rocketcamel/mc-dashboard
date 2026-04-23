mod auth;
mod env;
mod error;
mod routes;

use std::sync::Arc;

use axum::{Router, routing::get};
use console::style;
use openssh::{KnownHosts, Session};
use thiserror_ext::AsReport;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{env::Environment, error::Error};

pub struct AppState {
    pub ssh: Session,
    pub reqwest_client: reqwest::Client,
    pub environment: Environment,
}

async fn run() -> crate::error::Result<()> {
    let state = Arc::new(AppState {
        ssh: Session::connect("root@192.168.27.2", KnownHosts::Accept).await?,
        reqwest_client: reqwest::Client::new(),
        environment: Environment::load()?,
    });

    let app = Router::new()
        .route(
            "/",
            get(|| async { concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")) }),
        )
        .route("/api/backups", get(routes::get_backups::get_backups))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(|e| Error::bind_port(e, "0.0.0.0:8080"))?;
    tracing::info!("starting server on 0.0.0.0:8080");
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let tracing_env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(tracing_env_filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    if let Err(e) = run().await {
        eprintln!("{}: {}", style("error").red(), e.as_report())
    }
}
