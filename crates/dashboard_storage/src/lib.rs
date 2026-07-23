mod backup;
mod user;

pub mod session;

use std::sync::Arc;

use aws_config::BehaviorVersion;
use openssh::{KnownHosts, Session};

pub use backup::errors::StorageError;
pub use user::{UserError, UserErrorKind};

use crate::session::DynamoDBStore;

pub struct Environment {
    pub table_name: String,
}

pub const OPERATION_TIMEOUT: u64 = 500;
pub const NAMESPACE_NAME: &'static str = "minecraft";

pub struct Storage {
    pub ssh: Session,
    pub session_store: DynamoDBStore,
    pub client: aws_sdk_dynamodb::Client,
    pub environment: Arc<Environment>,
}

impl Storage {
    pub async fn create_storage(table_name: String) -> Result<Self, backup::errors::StorageError> {
        let config = aws_config::load_defaults(BehaviorVersion::v2026_01_12()).await;
        let client = aws_sdk_dynamodb::Client::new(&config);

        Ok(Self {
            client: client.clone(),
            ssh: Session::connect("root@192.168.27.2", KnownHosts::Accept).await?,
            session_store: DynamoDBStore::new(client, table_name.clone()),
            environment: Environment { table_name }.into(),
        })
    }
}
