use std::sync::Arc;

use axum::{
    Json,
    extract::{FromRef, FromRequest, State},
    response::IntoResponse,
};
use serde::Deserialize;
use strum::{AsRefStr, Display};

use crate::{
    AppState,
    auth::AuthUser,
    error::{Error, Result},
    k3s::backup_world as backup_world_k3s,
};

#[derive(Deserialize)]
pub struct BackupRequest {
    pub server_name: ServerName,
    pub backup_file_name: String,
}

#[derive(Deserialize, Display, AsRefStr, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ServerName {
    Main,
    Creative,
}

pub async fn backup_world(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
    request: BackupRequest,
) -> Result<impl IntoResponse> {
    backup_world_k3s(app_state, &request.server_name, &request.backup_file_name).await?;

    Ok(())
}

impl<S> FromRequest<S> for BackupRequest
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = Error;
    async fn from_request(
        req: axum::extract::Request,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let app_state = Arc::<AppState>::from_ref(state);
        let Json(body): Json<BackupRequest> = Json::from_request(req, state)
            .await
            .map_err(|_| Error::forbidden())?;
        let backups = app_state.storage.get_backups().await?;
        if !backups.iter().any(|b| b.filename == body.backup_file_name) {
            return Err(Error::not_found());
        }

        Ok(BackupRequest {
            server_name: body.server_name,
            backup_file_name: body.backup_file_name,
        })
    }
}
