use std::{env, sync::Arc};

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use serde::{Deserialize, Serialize};
use tower_sessions::{
    SessionStore,
    session::{Id, Record},
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
        Ok(None)
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        Ok(())
    }
}
