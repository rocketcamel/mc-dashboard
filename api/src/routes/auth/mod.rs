mod login;
mod register;

use std::sync::Arc;

use axum::{
    Json, Router,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use tower_sessions::Session;

use crate::{
    AppState,
    auth::AuthUser,
    error::{Error, Result},
};

pub async fn logout(session: Session) -> Result<impl IntoResponse> {
    session
        .flush()
        .await
        .map_err(|e| Error::internal(e.to_string()))?;
    Ok(Redirect::to("/login"))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login::login))
        .route("/register", post(register::register))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

pub async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(user)
}
