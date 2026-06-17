use std::sync::Arc;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use aws_sdk_dynamodb::{operation::put_item::PutItemError, types::AttributeValue};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use tower_sessions::Session;

use crate::{
    AppState,
    auth::User,
    error::{Error, Result},
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub auth: String,
}

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    session: Session,
    Json(request): Json<RegisterRequest>,
) -> Result<impl IntoResponse> {
    let hash = tokio::task::spawn_blocking({
        let auth = request.auth.clone();
        move || -> Result<String> {
            let salt = SaltString::generate(&mut rand_core::OsRng);
            Ok(Argon2::default()
                .hash_password(auth.as_bytes(), &salt)
                .map_err(|e| Error::internal(e.to_string()))?
                .to_string())
        }
    })
    .await
    .map_err(|e| Error::internal(e.to_string()))??;
    let result = app_state
        .dynamo
        .put_item()
        .table_name(&app_state.environment.table_name)
        .condition_expression("attribute_not_exists(pk)")
        .item(
            "pk",
            AttributeValue::S(format!("USER#{}", request.name.clone())),
        )
        .item("sk", AttributeValue::S("PROFILE".to_string()))
        .item("auth", AttributeValue::S(hash))
        .send()
        .await;
    if let Err(e) = result {
        return Err(match e.into_service_error() {
            PutItemError::ConditionalCheckFailedException(_) => Error::conflict(),
            other => Error::dynamo_d_b(format!("{other:?}")),
        });
    }

    session
        .insert("user", User { name: request.name })
        .await
        .map_err(|e| Error::internal(e.to_string()))?;
    Ok(StatusCode::OK)
}
