use aws_sdk_dynamodb::{operation::put_item::PutItemError, types::AttributeValue};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use types::Backup;

use crate::Storage;
use errors::StorageError;

#[derive(Serialize, Deserialize)]
pub struct LockItem {
    pub pk: String,
    pub sk: String,
    pub expiry_date: i64,
}

impl Storage {
    pub async fn get_backups(&self) -> Result<Vec<Backup>, StorageError> {
        let output = self
            .ssh
            .command("ls")
            .args(["-1s", "--block-size=1", "/backup/minecraft/"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(StorageError::ssh_command(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut backups: Vec<Backup> = stdout.lines().filter_map(parse_backup).collect();

        backups.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(backups)
    }

    pub async fn get_lock(&self) -> Result<bool, StorageError> {
        let result = self
            .client
            .get_item()
            .table_name(&self.environment.table_name)
            .key("pk", AttributeValue::S("OPERATION_LOCK".to_string()))
            .key("sk", AttributeValue::S("VALUE".to_string()))
            .send()
            .await?;

        let Some(item) = result.item else {
            return Ok(false);
        };

        let lock: LockItem = serde_dynamo::from_item(item)?;

        Ok(lock.expiry_date >= chrono::Utc::now().timestamp())
    }

    pub async fn aquire_lock(&self) -> Result<bool, StorageError> {
        let item = serde_dynamo::to_item(LockItem {
            pk: "OPERATION_LOCK".to_string(),
            sk: "VALUE".to_string(),
            expiry_date: chrono::Utc::now().timestamp() + 250,
        })?;

        let result = self
            .client
            .put_item()
            .table_name(&self.environment.table_name)
            .set_item(Some(item))
            .condition_expression("attribute_not_exists(pk) OR expiry_date < :now")
            .expression_attribute_values(
                ":now",
                AttributeValue::N(chrono::Utc::now().timestamp().to_string()),
            )
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => match e.into_service_error() {
                PutItemError::ConditionalCheckFailedException(_) => Ok(false),
                other => Err(StorageError::dynamo_db(format!("{other:?}"))),
            },
        }
    }

    pub async fn release_lock(&self) -> Result<(), StorageError> {
        self.client
            .delete_item()
            .table_name(&self.environment.table_name)
            .key("pk", AttributeValue::S("OPERATION_LOCK".to_string()))
            .key("sk", AttributeValue::S("VALUE".to_string()))
            .send()
            .await?;

        Ok(())
    }
}

fn parse_backup(line: &str) -> Option<Backup> {
    let (size_str, filename) = line.trim().split_once(' ')?;
    let name = filename.trim();

    if !name.starts_with("minecraft-main-") {
        return None;
    }

    let bytes = size_str.parse::<u64>().ok()?;
    let date_part = name.strip_prefix("minecraft-main-")?.split('.').next()?;
    let dt = NaiveDateTime::parse_from_str(date_part, "%Y%m%d-%H%M%S").ok()?;

    Some(Backup {
        filename: name.to_string(),
        bytes,
        date: dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
    })
}

pub mod errors {
    use std::fmt::Debug;

    use aws_sdk_dynamodb::error::SdkError;
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = StorageError))]
    pub enum StorageErrorKind {
        #[error("ssh command error: {0}")]
        SshCommand(String),
        #[error("ssh error")]
        Ssh(#[from] openssh::Error),

        #[error("dynamodb error: {0}")]
        DynamoDb(String),

        #[error("serde_dynamo error")]
        SerdeDynamo(#[from] serde_dynamo::Error),
    }

    impl<E: Debug> From<SdkError<E>> for StorageError {
        fn from(value: SdkError<E>) -> Self {
            StorageError::dynamo_db(format!("{value:?}"))
        }
    }
}
