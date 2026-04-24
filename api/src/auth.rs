use std::{env, sync::Arc};

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use axum::extract::FromRequestParts;
use serde::{Deserialize, Serialize};
use tower_sessions::{
    Session, SessionStore,
    session::{self, Id, Record},
    session_store,
};
use uuid::Uuid;

use crate::{env::Environment, error::Error};

#[derive(Serialize, Deserialize)]
#[allow(unused)]
pub struct UserItem {
    pub id: Uuid,
    pub name: String,
    pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionItem {
    pub pk: String,
    pub sk: String,
    pub data: String,
    pub expiry_date: i64,
}

#[derive(Debug, Clone)]
pub struct DynamoDBStore {
    client: Client,
    environment: Arc<Environment>,
}

#[async_trait]
impl SessionStore for DynamoDBStore {
    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let item = serde_dynamo::to_item(SessionItem {
            pk: format!("SESSION#{}", record.id.to_string()),
            sk: "SESSION".to_string(),
            data: serde_json::to_string(&record.data)
                .map_err(|e| session_store::Error::Encode(e.to_string()))?,
            expiry_date: record.expiry_date.unix_timestamp(),
        })
        .map_err(|e| session_store::Error::Encode(e.to_string()))?;

        self.client
            .put_item()
            .table_name(&self.environment.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let result = self
            .client
            .get_item()
            .table_name(&self.environment.table_name)
            .key(
                "pk",
                AttributeValue::S(format!("SESSION#{}", session_id.to_string())),
            )
            .key("sk", AttributeValue::S("SESSION".to_string()))
            .send()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;
        let Some(item) = result.item else {
            return Ok(None);
        };
        let session: SessionItem = serde_dynamo::from_item(item)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;

        Ok(Some(Record {
            id: *session_id,
            data: serde_json::from_str(&session.data)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?,
            expiry_date: time::OffsetDateTime::from_unix_timestamp(session.expiry_date)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?,
        }))
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        self.client
            .delete_item()
            .table_name(&self.environment.table_name)
            .key(
                "pk",
                AttributeValue::S(format!("SESSION#{}", session_id.to_string())),
            )
            .key("sk", AttributeValue::S("SESSION".to_string()))
            .send()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}

impl DynamoDBStore {
    pub fn new(client: Client, environment: Arc<Environment>) -> Self {
        Self {
            client,
            environment,
        }
    }
}

pub struct AuthUser(pub User);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::unauthorized())?;

        let user: Option<User> = session
            .get("user")
            .await
            .map_err(|_| Error::unauthorized())?;

        user.map(AuthUser).ok_or(Error::unauthorized())
    }
}
