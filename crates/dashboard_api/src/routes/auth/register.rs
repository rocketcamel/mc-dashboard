use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use tower_sessions::Session;

use super::errors::AuthRouteError;

use crate::{AppState, auth::AuthUser};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub auth: String,
}

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AuthRouteError> {
    let user = app_state
        .storage
        .create_user(&request.name, &request.auth)
        .await?;

    session
        .insert("user", user)
        .await
        .map_err(|_| AuthRouteError::session_insert())?;
    Ok(StatusCode::OK)
}
