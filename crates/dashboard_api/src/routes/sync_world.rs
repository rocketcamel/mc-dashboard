use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;
use types::World;

use crate::{AppState, auth::AuthUser, error::Result};

use dashboard_k3s::restore::sync_world as sync_world_k3s;

#[derive(Deserialize)]
pub struct SyncRequest {
    pub from_server_name: World,
    pub destination_server_name: World,
}

pub async fn sync_world(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
    Json(request): Json<SyncRequest>,
) -> Result<impl IntoResponse> {
    sync_world_k3s(
        app_state.storage.clone(),
        app_state.kubernetes.clone(),
        &request.from_server_name,
        &request.destination_server_name,
    )
    .await?;

    Ok(())
}
