use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use crate::{AppState, auth::AuthUser, error::Result};

#[derive(Serialize)]
pub struct StatusResponse {
    pub backing_up: bool,
}

pub async fn get_status(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
) -> Result<impl IntoResponse> {
    let backing_up = app_state.storage.get_lock().await?;

    Ok(Json(StatusResponse { backing_up }))
}
