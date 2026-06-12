use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api,
    api::{ListParams, LogParams},
};
use serde::Serialize;
use thiserror_ext::AsReport;

use crate::{
    AppState,
    routes::{World, logs::errors::StreamLogsErrorKind},
};

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    Log,
    Error,
}

#[derive(Serialize)]
pub struct LogStreamMessage<'a> {
    pub kind: MessageKind,
    pub data: &'a str,
}

fn to_message(kind: MessageKind, data: &str) -> Result<String, errors::StreamLogsError> {
    Ok(serde_json::to_string(&LogStreamMessage { kind, data })?)
}

async fn pod_name(world: &World, client: kube::Client) -> Result<String, errors::StreamLogsError> {
    let pods: Api<Pod> = Api::namespaced(client, "minecraft");

    for pod in pods
        .list(&ListParams::default().labels(&format!("app=minecraft-{world}")))
        .await?
    {
        if let Some(name) = pod.metadata.name {
            return Ok(name);
        } else {
            return Err(errors::StreamLogsError::no_name());
        }
    }

    Err(errors::StreamLogsError::null_pod())
}

pub async fn stream_logs(
    app_state: Arc<AppState>,
    world: World,
    socket: &mut WebSocket,
) -> Result<(), errors::StreamLogsError> {
    let pods: Api<Pod> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let name = pod_name(&world, app_state.kube.clone()).await?;

    let lp = LogParams {
        container: Some(format!("minecraft-{world}")),
        follow: true,
        ..Default::default()
    };
    let mut logs = pods.log_stream(&name, &lp).await?.lines();

    while let Some(log) = logs.try_next().await? {
        let message = to_message(MessageKind::Log, &log)?;
        if socket.send(Message::Text(message.into())).await.is_err() {
            return Ok(());
        }
    }

    Ok(())
}

async fn throw(socket: &mut WebSocket, why: &str) {
    let Ok(message) = to_message(MessageKind::Error, why) else {
        tracing::error!("websocket error: {why}");
        return;
    };

    let result = socket.send(Message::Text(message.into())).await;

    if let Err(e) = result {
        tracing::error!("websocket error: {}", e.as_report())
    }
}

pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Path(world): Path<World>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(async |mut socket| {
        let result = stream_logs(app_state, world, &mut socket).await;

        if let Err(e) = result {
            match e.inner() {
                StreamLogsErrorKind::NoName | StreamLogsErrorKind::NullPod => {
                    let why = e.as_report().to_string();
                    throw(&mut socket, &why).await;
                }
                _ => {
                    tracing::error!("websocket error: {}", e.as_report());
                }
            }
        }
    })
}

pub mod errors {
    use thiserror::Error;

    #[derive(Error, Debug, thiserror_ext::Box, thiserror_ext::Construct)]
    #[thiserror_ext(newtype(name = StreamLogsError))]
    pub enum StreamLogsErrorKind {
        #[error("pod has no name")]
        NoName,
        #[error("Server is offline")]
        NullPod,
        #[error("kube error")]
        Kube(#[from] kube::Error),

        #[error("error streaming logs")]
        Stream(#[from] std::io::Error),
        #[error("error serializing message")]
        Serialize(#[from] serde_json::Error),
    }
}
