use std::{collections::HashMap, sync::Arc};

use axum::{Json, response::IntoResponse};
use k8s_openapi::api::apps::v1::Deployment;
use kube::Api;
use serde::Serialize;

use crate::{AppState, error::Result};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Unknown,
}

pub async fn get_status(app_state: Arc<AppState>) -> Result<HashMap<String, ServerStatus>> {
    let deployments: Api<Deployment> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let mut statuses = HashMap::new();

    for world in ["main", "creative"] {
        let deployment = deployments.get(&format!("minecraft-{world}")).await?;

        let status = match deployment.status {
            Some(s) => {
                let replicas = s.replicas.unwrap_or(0);
                let ready = s.ready_replicas.unwrap_or(0);
                if replicas == 0 {
                    ServerStatus::Stopped
                } else if ready > 0 {
                    ServerStatus::Running
                } else {
                    ServerStatus::Starting
                }
            }
            _ => ServerStatus::Unknown,
        };

        statuses.insert(world.to_string(), status);
    }

    Ok(statuses)
}
