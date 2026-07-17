use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use dashboard_k3s::logging::{
    LogMessage, MessageKind, errors::LoggingErrorKind, snapshot_logs, stream_logs,
};
use futures::TryStreamExt;
use serde::Deserialize;
use thiserror_ext::AsReport;

use errors::StreamLogsError;
use types::World;

use crate::{AppState, auth::AuthUser, routes::logs::errors::StreamLogsErrorKind};

#[derive(Deserialize)]
pub struct QueryLogsRequest {
    pub world: World,
}

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

pub async fn get_logs(
    State(app_state): State<Arc<AppState>>,
    AuthUser(_): AuthUser,
    Query(request): Query<QueryLogsRequest>,
) -> Result<impl IntoResponse, StreamLogsError> {
    let logs = snapshot_logs(app_state.kubernetes.clone(), &request.world).await?;
    Ok(Json(logs))
}

pub async fn socket_handler(
    State(app_state): State<Arc<AppState>>,
    Path(world): Path<World>,
    AuthUser(_): AuthUser,
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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/query", get(get_logs))
        .route("/stream/{world}", get(socket_handler))
}

pub mod errors {
    use axum::Json;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use dashboard_k3s::logging;
    use dashboard_k3s::logging::errors::LoggingErrorKind;
    use serde::Serialize;
    use thiserror::Error;
    use thiserror_ext::{AsReport, Report};

    #[derive(Error, Debug, thiserror_ext::Box, thiserror_ext::Construct)]
    #[thiserror_ext(newtype(name = StreamLogsError))]
    pub enum StreamLogsErrorKind {
        #[error("error serializing message")]
        Serialize(#[from] serde_json::Error),

        #[error(transparent)]
        Logging(#[from] logging::LoggingError),
    }

    #[derive(Serialize)]
    pub struct ErrorResponse {
        pub errors: Vec<String>,
    }

    fn throw_internal(report: Report) -> StatusCode {
        tracing::error!("{}", report);
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn status(kind: &StreamLogsErrorKind, report: Report) -> StatusCode {
        match kind {
            StreamLogsErrorKind::Logging(e) => match e.inner() {
                _ => throw_internal(report),
            },

            _ => throw_internal(report),
        }
    }

    fn body(kind: &StreamLogsErrorKind) -> Option<ErrorResponse> {
        let mut response = ErrorResponse { errors: Vec::new() };

        let mut push = |data: String| {
            response.errors.push(data);
        };

        match kind {
            StreamLogsErrorKind::Logging(e) => match e.inner() {
                LoggingErrorKind::NullPod(_) => {
                    push(e.as_report().to_string());
                    Some(response)
                }

                _ => None,
            },

            _ => None,
        }
    }

    impl IntoResponse for StreamLogsError {
        fn into_response(self) -> axum::response::Response {
            let error_kind = self.inner();
            let status = status(error_kind, self.as_report());
            let body = body(error_kind);

            if let Some(body) = body {
                (status, Json(body)).into_response()
            } else {
                status.into_response()
            }
        }
    }
}
