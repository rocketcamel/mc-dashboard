use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use thiserror_ext::{Box, Construct};

#[derive(Error, Debug, Box, Construct)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("error binding port: {0}")]
    BindPort(String, #[source] std::io::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("ssh error: {0}")]
    Ssh(#[from] openssh::Error),
    #[error("ssh command failed: {0}")]
    SshCommand(String),
    #[error("error loading env")]
    Dotenv(#[from] dotenvy::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("serde dynamo error")]
    ConstructItem(#[from] serde_dynamo::Error),

    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("http error")]
    Http(#[from] reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self.inner() {
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            ErrorKind::Forbidden => StatusCode::FORBIDDEN.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
