use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::{AppState, auth::AuthUser, error::Result};

pub async fn get_backups(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
) -> Result<impl IntoResponse> {
    let backups = app_state.storage.get_backups().await?;
    Ok(Json(backups))
}
