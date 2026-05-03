use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use k8s_openapi::api::batch::v1::Job;
use kube::{Api, api::ListParams};
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
    let jobs: Api<Job> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let job_list = jobs.list(&ListParams::default()).await?;

    let backing_up = job_list.iter().any(|job| {
        job.status
            .as_ref()
            .map_or(false, |s| s.active.unwrap_or(0) > 0)
    });

    Ok(Json(StatusResponse { backing_up }))
}
