use std::{collections::HashMap, sync::Arc};

use k8s_openapi::{api::apps::v1::Deployment, serde::Serialize};
use kube::Api;

use crate::Kubernetes;

pub use errors::StatusError;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Unknown,
}

pub async fn get_status(
    kubernetes: Arc<Kubernetes>,
) -> Result<HashMap<String, ServerStatus>, StatusError> {
    let deployments: Api<Deployment> = Api::namespaced(kubernetes.client.clone(), "minecraft");
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

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = StatusError))]
    pub enum StatusErrorKind {
        #[error("kubernetes error")]
        Kubernetes(#[from] kube::Error),
    }
}
