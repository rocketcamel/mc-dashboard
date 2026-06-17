use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use dashboard_k3s::logging::{LogMessage, MessageKind, errors::LoggingErrorKind, stream_logs};
use futures::TryStreamExt;
use storage::World;
use thiserror_ext::AsReport;

use errors::StreamLogsError;

use crate::{AppState, routes::logs::errors::StreamLogsErrorKind};

fn to_message(message: &LogMessage) -> Result<String, StreamLogsError> {
    Ok(serde_json::to_string(message)?)
}

async fn throw(socket: &mut WebSocket, why: &str) {
    let Ok(message) = to_message(&LogMessage {
        kind: MessageKind::Error,
        data: why.to_string(),
    }) else {
        tracing::error!("websocket error: {why}");
        return;
    };

    let result = socket.send(Message::Text(message.into())).await;

    if let Err(e) = result {
        tracing::error!("websocket error: {}", e.as_report())
    }
}

async fn create_stream(
    socket: &mut WebSocket,
    app_state: Arc<AppState>,
    world: World,
) -> Result<(), StreamLogsError> {
    let mut stream = stream_logs(app_state.kubernetes.clone(), &world).await?;

    while let Some(log) = stream.try_next().await? {
        let message = to_message(&log)?;

        if socket.send(Message::Text(message.into())).await.is_err() {
            return Ok(());
        }
    }
    Ok(())
}

pub async fn handler(
    State(app_state): State<Arc<AppState>>,
    Path(world): Path<World>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade({
        async move |mut socket| {
            let result = create_stream(&mut socket, app_state, world).await;

            if let Err(e) = result {
                match e.inner() {
                    StreamLogsErrorKind::Logging(why) => match why.inner() {
                        LoggingErrorKind::NoName | LoggingErrorKind::NullPod(_) => {
                            let why = e.as_report().to_string();
                            throw(&mut socket, &why).await;
                        }
                        _ => {
                            tracing::error!("logging error: {}", e.as_report());
                        }
                    },

                    _ => {
                        tracing::error!("websocket error: {}", e.as_report());
                    }
                }
            }
        }
    })
}

pub mod errors {
    use dashboard_k3s::logging;
    use thiserror::Error;

    #[derive(Error, Debug, thiserror_ext::Box, thiserror_ext::Construct)]
    #[thiserror_ext(newtype(name = StreamLogsError))]
    pub enum StreamLogsErrorKind {
        #[error("error serializing message")]
        Serialize(#[from] serde_json::Error),

        #[error(transparent)]
        Logging(#[from] logging::LoggingError),
    }
}
