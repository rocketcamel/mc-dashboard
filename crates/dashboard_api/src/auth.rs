use axum::extract::FromRequestParts;

use errors::AuthError;
use tower_sessions::Session;
use types::User;

pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::unauthorized())?;

        let user: Option<User> = session
            .get("user")
            .await
            .map_err(|_| AuthError::unauthorized())?;

        user.map(AuthUser).ok_or(AuthError::unauthorized())
    }
}

pub mod errors {
    use axum::{http::StatusCode, response::IntoResponse};
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = AuthError))]
    pub enum AuthErrorKind {
        #[error("unauthorized")]
        Unauthorized,
    }

    impl IntoResponse for AuthError {
        fn into_response(self) -> axum::response::Response {
            match self.inner() {
                AuthErrorKind::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
                // _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }
}
