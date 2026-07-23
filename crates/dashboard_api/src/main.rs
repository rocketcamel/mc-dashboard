mod auth;
mod env;
mod error;
mod routes;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use console::style;
use dashboard_k3s::Kubernetes;
use storage::Storage;
use thiserror_ext::AsReport;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{env::Environment, error::Error};

pub struct AppState {
    pub storage: Arc<Storage>,
    pub kubernetes: Arc<Kubernetes>,
    pub reqwest_client: reqwest::Client,
    pub environment: Arc<Environment>,
}

async fn run() -> crate::error::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto");

    let environment = Arc::new(Environment::load()?);
    let storage = Arc::new(Storage::create_storage(environment.table_name.clone()).await?);
    let session_manager = SessionManagerLayer::new(storage.session_store.clone())
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

    let state = Arc::new(AppState {
        storage,
        kubernetes: Kubernetes::create_state().await?.into(),
        reqwest_client: reqwest::Client::new(),
        environment,
    });

    let app = Router::new()
        .route(
            "/api/version",
            get(|| async { concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")) }),
        )
        .route("/api/backups", get(routes::get_backups::get_backups))
        .route(
            "/api/backup_world",
            post(routes::backup_world::backup_world),
        )
        .route("/api/sync_world", post(routes::sync_world::sync_world))
        .route("/api/status", get(routes::status::get_status))
        .route("/api/world_status", get(routes::status::get_world_status))
        .route("/api/operation_log", get(routes::status::operation_log))
        .nest("/api/auth", routes::auth::router())
        .nest("/api/logs", routes::logs::router())
        .nest("/api/whitelist", routes::whitelist::router())
        .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
        .layer(session_manager)
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
