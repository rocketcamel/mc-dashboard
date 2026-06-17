use askama::Template;

pub mod logging;
pub mod restore;
pub mod status;

pub use restore::errors::KubernetesError;

pub struct Kubernetes {
    client: kube::Client,
}

impl Kubernetes {
    pub async fn create_state() -> Result<Self, restore::errors::KubernetesError> {
        Ok(Self {
            client: kube::Client::try_default().await?,
        })
    }
}

#[derive(Template)]
#[template(path = "restore-job.yaml", escape = "none")]
pub struct BackupJob<'a> {
    pub server_name: &'a str,
    pub backup_file_name: &'a str,
    pub should_op: bool,
}

#[derive(Template)]
#[template(path = "sync-job.yaml", escape = "none")]
pub struct SyncJob<'a> {
    pub from_server_name: &'a str,
}
