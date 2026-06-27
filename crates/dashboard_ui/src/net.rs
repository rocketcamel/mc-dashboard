use gloo::net::http::Request;
use types::{LoginRequest, User};
use web_sys::RequestCredentials;

use errors::NetError;

pub enum AuthStatus {
    Authenticated(User),
    Unauthenticated,
}

pub enum LoginStatus {
    Success(User),
    InvalidCredentials,
}

pub async fn login(username: String, auth: String) -> Result<LoginStatus, NetError> {
    let response = Request::get("/api/auth/login")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&LoginRequest { username, auth })?)?
        .send()
        .await?;

    match response.status() {
        200 => Ok(LoginStatus::Success(response.json().await?)),
        401 | 403 => Ok(LoginStatus::InvalidCredentials),
        _ => Err(NetError::internal()),
    }
}

pub async fn get_auth_status() -> Result<AuthStatus, NetError> {
    let response = Request::get("/api/auth/me")
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    match response.status() {
        200 => Ok(AuthStatus::Authenticated(response.json().await?)),
        401 | 403 => Ok(AuthStatus::Unauthenticated),
        _ => Err(NetError::internal()),
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
        #[error("internal server error")]
        Internal,
    }
}
