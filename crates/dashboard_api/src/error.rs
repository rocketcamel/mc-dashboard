use axum::{http::StatusCode, response::IntoResponse};
use storage::UserErrorKind;
use thiserror::Error;
use thiserror_ext::{AsReport, Box, Construct};

#[derive(Error, Debug, Box, Construct)]
#[thiserror_ext(newtype(name = Error))]
pub enum ErrorKind {
    #[error("error binding port: {0}")]
    BindPort(String, #[source] std::io::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
    // #[error("ssh error: {0}")]
    // Ssh(#[from] openssh::Error),
    #[error("error inserting user into session")]
    SessionInsert,

    #[error("forbidden")]
    Forbidden,
    #[error("resource not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Storage(#[from] storage::StorageError),
    #[error(transparent)]
    Kubernetes(#[from] dashboard_k3s::KubernetesError),
    #[error(transparent)]
    User(#[from] storage::UserError),
    #[error(transparent)]
    Status(#[from] dashboard_k3s::status::StatusError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self.inner() {
            ErrorKind::Forbidden => StatusCode::FORBIDDEN.into_response(),
            ErrorKind::NotFound => StatusCode::NOT_FOUND.into_response(),

            ErrorKind::User(error) => match error.inner() {
                UserErrorKind::InvalidCredentials => StatusCode::FORBIDDEN.into_response(),
                UserErrorKind::Null => StatusCode::FORBIDDEN.into_response(),

                _ => {
                    tracing::error!("{}", self.as_report());
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },

            _ => {
                tracing::error!("{}", self.as_report());
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
