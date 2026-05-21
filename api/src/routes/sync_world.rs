use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{
    AppState, auth::AuthUser, error::Result, k3s::sync_world as sync_world_k3s, routes::World,
};

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
        app_state,
        &request.from_server_name,
        &request.destination_server_name,
    )
    .await?;

    Ok(())
}
