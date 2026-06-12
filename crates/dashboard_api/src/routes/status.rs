use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use crate::{AppState, auth::AuthUser, error::Result, k3s::get_status as get_status_k3s};

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

pub async fn get_world_status(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
) -> Result<impl IntoResponse> {
    let statuses = get_status_k3s(app_state).await?;

    Ok(Json(statuses))
}
