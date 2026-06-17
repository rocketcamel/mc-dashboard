use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;
use tower_sessions::Session;

use super::errors::AuthRouteError;

use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub auth: String,
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthRouteError> {
    let user = app_state
        .storage
        .authenticate_user(&request.username, &request.auth)
        .await?;

    session
        .insert("user", user.clone())
        .await
        .map_err(|_| AuthRouteError::session_insert())?;

    Ok(Json(user))
}
