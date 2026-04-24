mod auth;
mod env;
mod error;
mod routes;

use std::sync::Arc;

use aws_config::BehaviorVersion;
use axum::{
    Router,
    routing::{get, post},
};
use console::style;
use openssh::{KnownHosts, Session};
use thiserror_ext::AsReport;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{auth::DynamoDBStore, env::Environment, error::Error};

pub struct AppState {
    pub ssh: Session,
    pub reqwest_client: reqwest::Client,
    pub environment: Arc<Environment>,
    pub dynamo: aws_sdk_dynamodb::Client,
}

async fn run() -> crate::error::Result<()> {
    let aws_config = aws_config::load_defaults(BehaviorVersion::v2026_01_12()).await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&aws_config);
    let environment = Arc::new(Environment::load()?);

    let dynamodb_store = DynamoDBStore::new(dynamodb_client.clone(), environment.clone());
    let session_manager = SessionManagerLayer::new(dynamodb_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

    let state = Arc::new(AppState {
        ssh: Session::connect("root@192.168.27.2", KnownHosts::Accept).await?,
        reqwest_client: reqwest::Client::new(),
        environment,
        dynamo: dynamodb_client,
    });

    let app = Router::new()
        .route(
            "/",
            get(|| async { concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")) }),
        )
        .route("/api/backups", get(routes::get_backups::get_backups))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/me", get(routes::auth::me))
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
