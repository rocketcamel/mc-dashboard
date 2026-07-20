use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use types::StatusResponse;

use crate::{AppState, auth::AuthUser, error::Result};

use dashboard_k3s::status::get_status as get_status_k3s;

pub async fn get_status(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
) -> Result<impl IntoResponse> {
    let backing_up = app_state.storage.get_lock().await?;
    Ok(Json(StatusResponse { backing_up }))
}

pub async fn get_world_status(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
) -> Result<impl IntoResponse> {
    let statuses = get_status_k3s(app_state.kubernetes.clone()).await?;
    Ok(Json(statuses))
}
