use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use tower_sessions::Session;
use types::LoginRequest;

use super::errors::AuthRouteError;

use crate::AppState;

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
