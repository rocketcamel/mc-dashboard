use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use thiserror_ext::{AsReport, Box, Construct};

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
    SerdeDynamo(#[from] serde_dynamo::Error),
    #[error("dynamodb error: {0}")]
    DynamoDB(String),
    #[error("error inserting user into session")]
    SessionInsert,

    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
    #[error("resource already exists")]
    Conflict,
    #[error("http error")]
    Http(#[from] reqwest::Error),
}

impl<E: std::fmt::Debug> From<aws_sdk_dynamodb::error::SdkError<E>> for Error {
    fn from(value: aws_sdk_dynamodb::error::SdkError<E>) -> Self {
        Error::dynamo_d_b(format!("{value:?}"))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self.inner() {
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            ErrorKind::Forbidden => StatusCode::FORBIDDEN.into_response(),
            ErrorKind::Conflict => StatusCode::CONFLICT.into_response(),
            ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),
            _ => {
                tracing::error!("{}", self.as_report());
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
