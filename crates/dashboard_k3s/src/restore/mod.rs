use std::{sync::Arc, time::Duration};

use askama::Template;
use k8s_openapi::api::{apps::v1::Deployment, batch::v1::Job};
use kube::{
    Api,
    api::{Patch, PatchParams, PostParams},
    runtime::{
        conditions,
        wait::{Condition, await_condition},
    },
};
use serde_json::json;
use storage::{NAMESPACE_NAME, OPERATION_TIMEOUT, Storage};

use errors::KubernetesError;
use types::World;

use crate::{BackupJob, Kubernetes, SyncJob};

fn should_op(server_name: &str) -> bool {
    match server_name {
        "creative" => true,
        _ => false,
    }
}

fn name(job: &Job) -> Result<String, KubernetesError> {
    job.metadata
        .name
        .clone()
        .ok_or(KubernetesError::internal(format!(
            "job: {job:?} has no name"
        )))
}

async fn scale_deployment(
    deployments: Api<Deployment>,
    server_name: &str,
    replicas: usize,
) -> Result<(), KubernetesError> {
    deployments
        .patch(
            &format!("minecraft-{server_name}"),
            &PatchParams::default(),
            &Patch::Merge(json!({"spec": {"replicas": replicas}})),
        )
        .await?;
    Ok(())
}

async fn operation_timeout(jobs: Api<Job>, name: &str, condition: impl Condition<Job>) -> bool {
    let condition = await_condition(jobs, name, condition);
    let timeout = tokio::time::timeout(Duration::from_secs(OPERATION_TIMEOUT), condition)
        .await
        .is_err();
    return timeout;
}

async fn create_backup_job(
    client: kube::Client,
    server_name: &str,
    backup_file_name: &str,
) -> Result<Job, KubernetesError> {
    let jobs: Api<Job> = Api::namespaced(client.clone(), NAMESPACE_NAME);
    let deployments: Api<Deployment> = Api::namespaced(client, NAMESPACE_NAME);

    let backup_job = BackupJob {
        server_name: server_name.as_ref(),
        backup_file_name,
        should_op: should_op(server_name.as_ref()),
    }
    .render()?;

    let job: Job = serde_yaml::from_str(&backup_job)?;

    scale_deployment(deployments, server_name.as_ref(), 0).await?;
    jobs.create(&PostParams::default(), &job).await?;

    Ok(job)
}

async fn create_sync_job(
    client: kube::Client,
    from_server_name: &str,
) -> Result<Job, KubernetesError> {
    let jobs: Api<Job> = Api::namespaced(client, NAMESPACE_NAME);

    let sync_job = SyncJob {
        from_server_name: from_server_name.as_ref(),
    }
    .render()?;

    let job: Job = serde_yaml::from_str(&sync_job)?;
    jobs.create(&PostParams::default(), &job).await?;

    Ok(job)
}

async fn backup_task(
    storage: Arc<Storage>,
    kubernetes: Arc<Kubernetes>,
    job_name: String,
    server_name: String,
) -> Result<(), KubernetesError> {
    let jobs: Api<Job> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let timeout = operation_timeout(jobs, &job_name, conditions::is_job_completed()).await;

    if timeout {
        tracing::error!("backup job {job_name} failed");
    }

    let deployments: Api<Deployment> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);

    scale_deployment(deployments, server_name.as_ref(), 1).await?;

    storage.release_lock().await?;
    Ok(())
}

pub async fn backup_world(
    storage: Arc<Storage>,
    kubernetes: Arc<Kubernetes>,
    server_name: &World,
    backup_file_name: &str,
) -> Result<(), KubernetesError> {
    if !storage.aquire_lock().await? {
        return Err(KubernetesError::conflict());
    }

    let job = create_backup_job(
        kubernetes.client.clone(),
        server_name.as_ref(),
        backup_file_name,
    )
    .await?;

    tokio::spawn({
        let job_name = name(&job)?;
        let server_name = server_name.to_string();

        async move {
            let result = backup_task(storage, kubernetes, job_name, server_name).await;

            if let Err(e) = result {
                tracing::error!("backup task failed: {e:?}")
            }
        }
    });

    Ok(())
}

async fn sync_task(
    storage: Arc<Storage>,
    kubernetes: Arc<Kubernetes>,
    job_name: String,
    from_server_name: String,
    dest_server_name: String,
) -> Result<(), KubernetesError> {
    let jobs: Api<Job> = Api::namespaced(kubernetes.client.clone(), NAMESPACE_NAME);
    let timeout = operation_timeout(jobs, &job_name, conditions::is_job_completed()).await;

    if timeout {
        return Err(KubernetesError::internal(format!(
            "sync job: {job_name} timed out"
        )));
    }

    let job = create_backup_job(
        kubernetes.client.clone(),
        &dest_server_name,
        &format!("{from_server_name}-sync.tar.gz"),
    )
    .await?;

    let job_name = name(&job)?;

    backup_task(storage, kubernetes, job_name, dest_server_name).await?;
    Ok(())
}

pub async fn sync_world(
    storage: Arc<Storage>,
    kubernetes: Arc<Kubernetes>,
    from_server_name: &World,
    dest_server_name: &World,
) -> Result<(), KubernetesError> {
    if !storage.aquire_lock().await? {
        return Err(KubernetesError::conflict());
    }

    let job = create_sync_job(kubernetes.client.clone(), from_server_name.as_ref()).await?;

    tokio::spawn({
        let job_name = name(&job)?.clone();
        let from_server_name = from_server_name.to_string();
        let dest_server_name = dest_server_name.to_string();

        async move {
            let result = sync_task(
                storage,
                kubernetes,
                job_name,
                from_server_name,
                dest_server_name,
            )
            .await;

            if let Err(e) = result {
                tracing::error!("sync task failed: {e:?}")
            }
        }
    });

    Ok(())
}

pub mod errors {
    use thiserror::Error;
    use thiserror_ext::{Box, Construct};

    #[derive(Error, Construct, Box, Debug)]
    #[thiserror_ext(newtype(name = KubernetesError))]
    pub enum KubernetesErrorKind {
        #[error("a backup job is already running")]
        Conflict,
        #[error("kubernetes error")]
        Kubernetes(#[from] kube::Error),
        #[error("internal error")]
        Internal(String),
        #[error(transparent)]
        Storage(#[from] storage::StorageError),
        #[error("askama template error")]
        Template(#[from] askama::Error),
        #[error("yaml error")]
        Yaml(#[from] serde_yaml::Error),
    }
}
