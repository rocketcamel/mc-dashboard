mod error;

use axum::{Router, routing::get};
use console::style;
use thiserror_ext::AsReport;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::Error;

async fn run() -> crate::error::Result<()> {
    let app = Router::new().route(
        "/",
        get(|| async { concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")) })
            .layer(TraceLayer::new_for_http()),
    );
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
