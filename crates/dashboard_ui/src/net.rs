use std::collections::HashMap;

use gloo::net::http::Request;
use types::{LoginRequest, ServerStatus, User};
use web_sys::RequestCredentials;

use errors::{NetError, NetErrorKind};

pub enum AuthStatus {
    Authenticated(User),
    Unauthenticated,
}

pub enum LoginStatus {
    Success(User),
    InvalidCredentials,
}

fn error_for_status(status: u16) -> Option<NetError> {
    match status {
        401 => Some(NetError::unauthenticated()),
        403 => Some(NetError::forbidden()),
        500..=599 => Some(NetError::internal()),
        _ => None,
    }
}

pub async fn world_status() -> Result<Vec<HashMap<String, ServerStatus>>, NetError> {
    let response = Request::get("/api/world_status")
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => Err(err),
        _ => Ok(response.json().await?),
    }
}

pub async fn login(username: String, auth: String) -> Result<LoginStatus, NetError> {
    let response = Request::post("/api/auth/login")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&LoginRequest { username, auth })?)?
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => match err.inner() {
            NetErrorKind::Unauthenticated | NetErrorKind::Forbidden => {
                Ok(LoginStatus::InvalidCredentials)
            }
            _ => Err(err),
        },

        _ => Ok(LoginStatus::Success(response.json().await?)),
    }
}

pub async fn logout() -> Result<(), NetError> {
    let response = Request::post("/api/auth/logout").send().await?;

    match response.status() {
        200 => Ok(()),
        _ => Err(NetError::internal()),
    }
}

pub async fn get_auth_status() -> Result<AuthStatus, NetError> {
    let response = Request::get("/api/auth/me")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    match error_for_status(response.status()) {
        Some(err) => match err.inner() {
            NetErrorKind::Unauthenticated | NetErrorKind::Forbidden => {
                Ok(AuthStatus::Unauthenticated)
            }
            _ => Err(err),
        },

        _ => Ok(AuthStatus::Authenticated(response.json().await?)),
    }
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = NetError))]
    pub enum NetErrorKind {
        #[error("network error")]
        Network(#[from] gloo::net::Error),
        #[error("json error")]
        Json(#[from] serde_json::Error),
        #[error("unauthenticated")]
        Unauthenticated,
        #[error("forbidden")]
        Forbidden,

        #[error("internal server error")]
        Internal,
    }
}
