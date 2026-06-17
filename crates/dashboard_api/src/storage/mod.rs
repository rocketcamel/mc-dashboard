mod backup;

use std::sync::Arc;

use openssh::Session;
use serde::Serialize;

use crate::env::Environment;

pub struct Storage {
    pub ssh: Session,
    pub dynamo: aws_sdk_dynamodb::Client,
    pub environment: Arc<Environment>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Backup {
    pub filename: String,
    pub bytes: u64,
    pub date: String,
}
