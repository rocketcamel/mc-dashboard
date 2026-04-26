use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{AppState, auth::AuthUser, error::Result, k3s::backup_world as backup_world_k3s};

#[derive(Deserialize)]
pub struct BackupRequest {
    pub server_name: String,
    pub backup_file_name: String,
}

pub async fn backup_world(
    State(app_state): State<Arc<AppState>>,
    // AuthUser(_): AuthUser,
    Json(request): Json<BackupRequest>,
) -> Result<impl IntoResponse> {
    backup_world_k3s(app_state, &request.server_name, &request.backup_file_name).await?;

    Ok(())
}
