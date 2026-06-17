use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use aws_sdk_dynamodb::types::AttributeValue;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    AppState,
    auth::{User, UserItem},
    error::{Error, Result},
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub auth: String,
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse> {
    let result = app_state
        .dynamo
        .get_item()
        .table_name(&app_state.environment.table_name)
        .key(
            "pk",
            AttributeValue::S(format!("USER#{}", request.username.clone())),
        )
        .key("sk", AttributeValue::S("PROFILE".to_string()))
        .send()
        .await?;
    let Some(item) = result.item else {
        return Err(Error::unauthorized());
    };
    let user: UserItem = serde_dynamo::from_item(item)?;

    tokio::task::spawn_blocking({
        let auth_input = request.auth;
        let auth_hash = user.auth.clone();
        move || {
            let parsed_hash = PasswordHash::new(&auth_hash).map_err(|_| Error::unauthorized())?;
            Argon2::default()
                .verify_password(auth_input.as_bytes(), &parsed_hash)
                .map_err(|_| Error::unauthorized())
        }
    })
    .await
    .map_err(|e| Error::internal(e.to_string()))??;
    let user = User {
        name: request.username,
    };
    session
        .insert("user", user.clone())
        .await
        .map_err(|_| Error::session_insert())?;

    Ok(Json(user))
}
