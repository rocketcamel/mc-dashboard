mod login;
#[allow(unused)]
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

pub mod errors {
    use axum::{body::Body, http::Response, response::IntoResponse};
    use reqwest::StatusCode;
    use storage::UserErrorKind;
    use thiserror::Error;
    use thiserror_ext::{AsReport, Box, Construct, Report};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = AuthRouteError))]
    pub enum AuthRouteErrorKind {
        #[error("error inserting user into session")]
        SessionInsert,

        #[error(transparent)]
        User(#[from] storage::UserError),
    }

    fn throw_internal(report: Report) -> Response<Body> {
        tracing::error!("{}", report);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }

    impl IntoResponse for AuthRouteError {
        fn into_response(self) -> axum::response::Response {
            match self.inner() {
                AuthRouteErrorKind::User(e) => match e.inner() {
                    UserErrorKind::InvalidCredentials => StatusCode::FORBIDDEN.into_response(),
                    UserErrorKind::Conflict => StatusCode::CONFLICT.into_response(),
                    UserErrorKind::Null => StatusCode::FORBIDDEN.into_response(),

                    _ => throw_internal(self.as_report()),
                },

                _ => throw_internal(self.as_report()),
            }
        }
    }
}
