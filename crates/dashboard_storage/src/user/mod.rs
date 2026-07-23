use std::collections::HashMap;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use aws_sdk_dynamodb::{operation::put_item::PutItemError, types::AttributeValue};
use serde::{Deserialize, Serialize};
use types::User;

use crate::Storage;

pub use errors::{UserError, UserErrorKind};

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct UserItem {
    pub pk: String,
    pub sk: String,
    pub auth: String,
}

fn user_item(item: Option<HashMap<String, AttributeValue>>) -> Result<UserItem, UserError> {
    let Some(item) = item else {
        return Err(UserError::null());
    };

    let user: UserItem = serde_dynamo::from_item(item)?;
    Ok(user)
}

fn create_hash(auth: &str) -> Result<String, UserError> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    Ok(Argon2::default()
        .hash_password(auth.as_bytes(), &salt)
        .map_err(|e| UserError::internal(e.to_string()))?
        .to_string())
}

impl Storage {
    async fn retrieve_user(&self, username: &str) -> Result<UserItem, UserError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.environment.table_name)
            .key("pk", AttributeValue::S(format!("USER#{}", username)))
            .key("sk", AttributeValue::S("PROFILE".to_string()))
            .send()
            .await?;
        return user_item(result.item);
    }

    pub async fn create_user(&self, username: &str, auth: &str) -> Result<User, UserError> {
        let hash = tokio::task::spawn_blocking({
            let auth = auth.to_string();
            move || create_hash(&auth)
        })
        .await
        .map_err(|e| UserError::internal(e.to_string()))??;

        let item = serde_dynamo::to_item(UserItem {
            pk: format!("USER#{username}"),
            sk: "PROFILE".to_string(),
            auth: hash,
        })?;

        let result = self
            .client
            .put_item()
            .table_name(&self.environment.table_name)
            .condition_expression("attribute_not_exists(pk)")
            .set_item(Some(item))
            .send()
            .await;

        if let Err(e) = result {
            match e.into_service_error() {
                PutItemError::ConditionalCheckFailedException(_) => {
                    return Err(UserError::conflict());
                }
                other => return Err(UserError::dynamo_db(format!("{other:?}"))),
            }
        }

        Ok(User {
            name: username.to_string(),
        })
    }

    pub async fn authenticate_user(&self, username: &str, auth: &str) -> Result<User, UserError> {
        let user = self.retrieve_user(username).await?;

        tokio::task::spawn_blocking({
            let auth = auth.to_string();

            move || {
                let hash = PasswordHash::new(&user.auth)
                    .map_err(|e| UserError::internal(e.to_string()))?;
                Argon2::default()
                    .verify_password(auth.as_bytes(), &hash)
                    .map_err(|_| UserError::invalid_credentials())
            }
        })
        .await
        .map_err(|e| UserError::internal(e.to_string()))??;

        Ok(User {
            name: username.to_string(),
        })
    }
}

pub mod errors {
    use std::fmt::Debug;

    use aws_sdk_dynamodb::error::SdkError;
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = UserError))]
    pub enum UserErrorKind {
        #[error("dynamodb error: {0}")]
        DynamoDb(String),
        #[error("user doesnt exist")]
        Null,
        #[error("invalid credentials")]
        InvalidCredentials,
        #[error("conflicting users")]
        Conflict,
        #[error("serde_dynamo error")]
        SerdeDynamo(#[from] serde_dynamo::Error),
        #[error("internal error")]
        Internal(String),
    }

    impl<E: Debug> From<SdkError<E>> for UserError {
        fn from(value: SdkError<E>) -> Self {
            Self::dynamo_db(format!("{value:?}"))
        }
    }
}
