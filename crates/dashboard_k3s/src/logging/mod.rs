use std::{pin::Pin, sync::Arc};

use futures::{AsyncBufReadExt, Stream, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api,
    api::{ListParams, LogParams},
};
use serde::Serialize;
use storage::{NAMESPACE_NAME, World};

pub use errors::LoggingError;

use crate::Kubernetes;

type Cursor = usize;

pub type LogStream = Pin<Box<dyn Stream<Item = Result<LogMessage, LoggingError>> + Send>>;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    Log,
    Error,
}

#[derive(Serialize)]
pub struct LogMessage {
    pub kind: MessageKind,
    pub data: String,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub data: Vec<LogMessage>,
}

async fn pod_name(world: &World, client: kube::Client) -> Result<String, LoggingError> {
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE_NAME);

    for pod in pods
        .list(&ListParams::default().labels(&format!("app=minecraft-{world}")))
        .await?
    {
        if let Some(name) = pod.metadata.name {
            return Ok(name);
        } else {
            return Err(LoggingError::no_name());
        }
    }

    Err(LoggingError::null_pod(world.as_ref()))
}

pub async fn snapshot_logs(
    kubernetes: Arc<Kubernetes>,
    world: &World,
) -> Result<Snapshot, LoggingError> {
    let pods: Api<Pod> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let name = pod_name(&world, kubernetes.client.clone()).await?;

    let lp = LogParams {
        container: Some(format!("minecraft-{world}")),
        tail_lines: Some(300),
        ..Default::default()
    };

    let logs: Vec<LogMessage> = pods
        .logs(&name, &lp)
        .await?
        .split("\n")
        .map(|data| LogMessage {
            kind: MessageKind::Log,
            data: data.to_string(),
        })
        .collect();

    Ok(Snapshot { data: logs })
}

pub async fn stream_logs(
    kubernetes: Arc<Kubernetes>,
    world: &World,
) -> Result<LogStream, LoggingError> {
    let pods: Api<Pod> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let name = pod_name(&world, kubernetes.client.clone()).await?;

    let lp = LogParams {
        container: Some(format!("minecraft-{world}")),
        follow: true,
        since_time: Some(k8s_openapi::jiff::Timestamp::now()),
        ..Default::default()
    };

    let stream = pods
        .log_stream(&name, &lp)
        .await?
        .lines()
        .map_err(LoggingError::from)
        .map_ok(|data| LogMessage {
            kind: MessageKind::Log,
            data,
        });

    Ok(Box::pin(stream))
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = LoggingError))]
    pub enum LoggingErrorKind {
        #[error("pod does not exist for {0}")]
        NullPod(String),
        #[error("pod name is null")]
        NoName,
        #[error("kubernetes error")]
        Kubernetes(#[from] kube::Error),

        #[error("json error")]
        Json(#[from] serde_json::Error),

        #[error("io error")]
        Io(#[from] std::io::Error),
    }
}
