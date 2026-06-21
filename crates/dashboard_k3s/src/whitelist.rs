use std::sync::Arc;

use k8s_openapi::api::core::v1::Pod;
use kube::{
    Api,
    api::{AttachParams, AttachedProcess, ListParams},
};
use serde::{Deserialize, Serialize};
use storage::{NAMESPACE_NAME, World};

pub use errors::WhitelistError;
use tokio::io::AsyncReadExt;

use crate::Kubernetes;

#[derive(Serialize, Deserialize)]
pub struct WhitelistEntry {
    #[serde(rename(serialize = "username"))]
    pub name: String,
    pub uuid: String,
}

fn container_name(world: &World) -> String {
    return format!("minecraft-{world}");
}

async fn pod_name(world: &World, client: kube::Client) -> Result<String, WhitelistError> {
    let pods: Api<Pod> = Api::namespaced(client, NAMESPACE_NAME);

    for pod in pods
        .list(&ListParams::default().labels(&format!("app=minecraft-{world}")))
        .await?
    {
        if let Some(name) = pod.metadata.name {
            return Ok(name);
        } else {
            return Err(WhitelistError::no_name());
        }
    }

    Err(WhitelistError::null_pod(world.as_ref()))
}

async fn proc_read(proc: &mut AttachedProcess) -> Result<(String, String), WhitelistError> {
    let mut out = String::new();
    let mut stdout = proc.stdout().unwrap();
    stdout.read_to_string(&mut out).await?;

    let mut err = String::new();
    let mut stderr = proc.stderr().unwrap();
    stderr.read_to_string(&mut err).await?;

    Ok((out, err))
}

pub async fn whitelist_add(
    kubernetes: Arc<Kubernetes>,
    username: &str,
    world: &World,
) -> Result<(), WhitelistError> {
    let pods: Api<Pod> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let name = pod_name(world, kubernetes.client.clone()).await?;

    let ap = AttachParams::default()
        .container(container_name(world))
        .stdout(true)
        .stderr(true);

    let mut proc = pods
        .exec(&name, vec!["rcon-cli", "whitelist", "add", username], &ap)
        .await?;

    let (out, err) = proc_read(&mut proc).await?;

    let status = proc
        .take_status()
        .ok_or(WhitelistError::null_status())?
        .await
        .ok_or(WhitelistError::command_unknown(&err))?;

    if status.status.as_deref() != Some("Success") {
        return Err(WhitelistError::command(format!("{status:?}"), err, out));
    }
    Ok(())
}

pub async fn whitelist_get(
    kubernetes: Arc<Kubernetes>,
    world: &World,
) -> Result<Vec<WhitelistEntry>, WhitelistError> {
    let pods: Api<Pod> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let name = pod_name(world, kubernetes.client.clone()).await?;

    let ap = AttachParams::default()
        .container(container_name(world))
        .stdout(true)
        .stderr(true);

    let mut proc = pods
        .exec(&name, vec!["sh", "-c", "cat /data/whitelist.json"], &ap)
        .await?;

    let (out, err) = proc_read(&mut proc).await?;

    let status = proc
        .take_status()
        .ok_or(WhitelistError::null_status())?
        .await
        .ok_or(WhitelistError::command_unknown(&err))?;

    if status.status.as_deref() != Some("Success") {
        return Err(WhitelistError::command(format!("{status:?}"), err, out));
    }
    Ok(serde_json::from_str(&out)?)
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = WhitelistError))]
    pub enum WhitelistErrorKind {
        #[error("pod does not exist for {0}")]
        NullPod(String),
        #[error("command execution produced no status")]
        NullStatus,
        #[error("pod name is null")]
        NoName,
        #[error("kubernetes error")]
        Kubernetes(#[from] kube::Error),
        #[error("json error")]
        Json(#[from] serde_json::Error),

        #[error("command error: {message}, stderr={stderr}, stdout={stdout}")]
        Command {
            message: String,
            stderr: String,
            stdout: String,
        },
        #[error("unknown command error: {0}")]
        CommandUnknown(String),

        #[error("io error")]
        Io(#[from] std::io::Error),
    }
}
