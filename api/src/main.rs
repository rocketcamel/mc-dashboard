mod auth;
mod env;
mod error;
mod k3s;
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
    pub kube: kube::Client,
}

async fn run() -> crate::error::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls crypto");

    let aws_config = aws_config::load_defaults(BehaviorVersion::v2026_01_12()).await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&aws_config);
    let environment = Arc::new(Environment::load()?);

    let dynamodb_store = DynamoDBStore::new(dynamodb_client.clone(), environment.clone());
    let session_manager = SessionManagerLayer::new(dynamodb_store)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));
    let kube_client = kube::Client::try_default()
        .await
        .map_err(|e| Error::kube_connect(e))?;

    let state = Arc::new(AppState {
        ssh: Session::connect("root@192.168.27.2", KnownHosts::Accept).await?,
        reqwest_client: reqwest::Client::new(),
        environment,
        dynamo: dynamodb_client,
        kube: kube_client,
    });

    let app = Router::new()
        .route(
            "/",
            get(|| async { concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")) }),
        )
        .route("/api/backups", get(routes::get_backups::get_backups))
        .route(
            "/api/backup_world",
            post(routes::backup_world::backup_world),
        )
        .route("/api/status", get(routes::status::get_status))
        .nest("/api/auth", routes::auth::router())
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
