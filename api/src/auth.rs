use std::{env, sync::Arc};

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use serde::{Deserialize, Serialize};
use tower_sessions::{
    SessionStore,
    session::{self, Id, Record},
    session_store,
};
use uuid::Uuid;

use crate::env::Environment;

#[derive(Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    name: String,
    auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionItem {
    pk: String,
    sk: String,
    data: String,
    expiry_date: i64,
}

#[derive(Debug)]
pub struct DynamoDBStore {
    client: Arc<Client>,
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
