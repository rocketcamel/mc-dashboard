use std::{sync::Arc, time::Duration};

use askama::Template;
use k8s_openapi::api::{apps::v1::Deployment, batch::v1::Job};
use kube::{
    Api,
    api::{Patch, PatchParams, PostParams},
    runtime::{conditions, wait::await_condition},
};
use serde_json::json;

use crate::{
    AppState,
    error::{Error, Result},
    k3s::{BackupJob, SyncJob},
    routes::backup_world::ServerName,
};

pub async fn sync_world(
    app_state: Arc<AppState>,
    from_server_name: &ServerName,
    destination_server_name: &ServerName,
) -> Result<()> {
    if !app_state.storage.aquire_lock().await? {
        return Err(Error::conflict());
    }

    let jobs: Api<Job> = Api::namespaced(app_state.kube.clone(), "minecraft");

    let sync_job = SyncJob {
        from_server_name: from_server_name.as_ref(),
    }
    .render()?;
    let job: Job = serde_yaml::from_str(&sync_job)?;
    jobs.create(&PostParams::default(), &job).await?;

    tokio::spawn({
        let job_name = job
            .metadata
            .name
            .ok_or(Error::internal("job has no name"))?
            .clone();
        let from_server_name = from_server_name.clone();
        let destination_server_name = destination_server_name.clone();
        async move {
            if let Err(e) = sync_handler(
                app_state.clone(),
                jobs,
                job_name,
                from_server_name,
                destination_server_name,
            )
            .await
            {
                tracing::error!("sync failed: {e:?}")
            }
            if let Err(e) = app_state.storage.release_lock().await {
                tracing::error!("error releasing lock: {e:?}")
            }
        }
    });
    Ok(())
}

async fn sync_handler(
    app_state: Arc<AppState>,
    jobs: Api<Job>,
    job_name: String,
    from_server_name: ServerName,
    destination_server_name: ServerName,
) -> Result<()> {
    let deployments: Api<Deployment> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let cond = await_condition(jobs.clone(), &job_name, conditions::is_job_completed());
    tokio::time::timeout(Duration::from_secs(250), cond)
        .await
        .map_err(|_| Error::internal("sync job timed out: {job_name}"))?
        .map_err(|e| Error::internal(format!("watch error: {e:?}")))?;

    scale_deployment(
        deployments.clone(),
        &format!("minecraft-{destination_server_name}"),
        0,
    )
    .await?;

    let backup_job = BackupJob {
        server_name: destination_server_name.as_ref(),
        backup_file_name: &format!("{from_server_name}-sync.tar.gz"),
        should_op: destination_server_name.as_ref() == "creative",
    }
    .render()?;
    let job: Job = serde_yaml::from_str(&backup_job)?;
    jobs.create(&PostParams::default(), &job).await?;
    let job_name = job
        .metadata
        .name
        .ok_or(Error::internal("job has no name"))?;
    let cond = await_condition(jobs, &job_name, conditions::is_job_completed());
    let timed_out = tokio::time::timeout(Duration::from_secs(250), cond)
        .await
        .is_err();

    if timed_out {
        tracing::error!("backup job timed out: {job_name}")
    }

    scale_deployment(
        deployments,
        &format!("minecraft-{destination_server_name}"),
        1,
    )
    .await?;

    Ok(())
}

async fn scale_deployment(deployments: Api<Deployment>, name: &str, replicas: u16) -> Result<()> {
    deployments
        .patch(
            name,
            &PatchParams::default(),
            &Patch::Merge(json!({ "spec": { "replicas": replicas }})),
        )
        .await
        .map_err(|e| Error::internal(format!("scaling deployment failed: {e:?}")))?;
    Ok(())
}
