use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};

pub use errors::WhitelistRouteError;
use serde::Deserialize;
use storage::World;

use crate::{AppState, auth::AuthUser};

use dashboard_k3s::whitelist::{
    whitelist_add as whitelist_add_k3s, whitelist_get as whitelist_get_k3s,
};

#[derive(Deserialize)]
pub struct WhitelistRequest {
    pub username: String,
    pub world: World,
}

#[derive(Deserialize)]
pub struct WhitelistGetRequest {
    pub world: World,
}

pub async fn whitelist_add(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
    Json(request): Json<WhitelistRequest>,
) -> Result<impl IntoResponse, WhitelistRouteError> {
    whitelist_add_k3s(
        app_state.kubernetes.clone(),
        &request.username,
        &request.world,
    )
    .await?;
    Ok(())
}

pub async fn whitelist_get(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
    Query(request): Query<WhitelistGetRequest>,
) -> Result<impl IntoResponse, WhitelistRouteError> {
    let entries = whitelist_get_k3s(app_state.kubernetes.clone(), &request.world).await?;
    Ok(Json(entries))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/add", post(whitelist_add))
        .route("/get", get(whitelist_get))
}

pub mod errors {
    use axum::{Json, http::StatusCode, response::IntoResponse};
    use dashboard_k3s::whitelist::{self, errors::WhitelistErrorKind};
    use serde::Serialize;
    use thiserror::Error;
    use thiserror_ext::{AsReport, Box, Construct, Report};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = WhitelistRouteError))]
    pub enum WhitelistRouteErrorKind {
        #[error(transparent)]
        Whitelist(#[from] whitelist::WhitelistError),
    }

    #[derive(Serialize)]
    pub struct ErrorResponse {
        pub errors: Vec<String>,
    }

    fn status(kind: &WhitelistRouteErrorKind) -> StatusCode {
        match kind {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn throw_body(report: Report) -> Option<ErrorResponse> {
        tracing::error!("{}", report);
        None
    }

    fn body(kind: &WhitelistRouteErrorKind) -> Option<ErrorResponse> {
        let mut response = ErrorResponse { errors: Vec::new() };

        let mut push = |data: String| {
            response.errors.push(data);
        };

        match kind {
            WhitelistRouteErrorKind::Whitelist(e) => match e.inner() {
                WhitelistErrorKind::Command {
                    message,
                    stderr,
                    stdout,
                } => {
                    push(message.clone());
                    tracing::error!(
                        "whitelist command execution error: {stderr}, {stdout}, {}",
                        e.as_report()
                    );
                    Some(response)
                }

                _ => throw_body(e.as_report()),
            },
        }
    }

    impl IntoResponse for WhitelistRouteError {
        fn into_response(self) -> axum::response::Response {
            let error_kind = self.inner();

            let status = status(error_kind);
            let body = body(error_kind);

            if let Some(body) = body {
                (status, Json(body)).into_response()
            } else {
                status.into_response()
            }
        }
    }
}
