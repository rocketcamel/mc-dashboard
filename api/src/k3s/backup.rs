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
    k3s::BackupJob,
};

pub async fn backup_world(
    app_state: Arc<AppState>,
    server_name: &str,
    backup_file_name: &str,
) -> Result<()> {
    let jobs: Api<Job> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let deployments: Api<Deployment> = Api::namespaced(app_state.kube.clone(), "minecraft");
    let backup_job = BackupJob {
        server_name,
        backup_file_name,
        should_op: should_op(server_name),
    }
    .render()?;
    println!("{backup_job}");
    let job: Job = serde_yaml::from_str(&backup_job)?;
    deployments
        .patch(
            &format!("minecraft-{server_name}"),
            &PatchParams::default(),
            &Patch::Merge(&json!({"spec": {"replicas": 0}})),
        )
        .await?;
    jobs.create(&PostParams::default(), &job).await?;

    tokio::spawn({
        let jobs = jobs.clone();
        let job_name = job
            .metadata
            .name
            .ok_or(Error::internal("job has no name"))?
            .clone();
        let deployments = deployments.clone();
        let server_name = server_name.to_string();
        async move {
            let cond = await_condition(jobs, &job_name, conditions::is_job_completed());
            match tokio::time::timeout(Duration::from_secs(600), cond).await {
                Ok(Ok(_)) => {
                    let result = deployments
                        .patch(
                            &format!("minecraft-{server_name}"),
                            &PatchParams::default(),
                            &Patch::Merge(&json!({"spec": {"replicas": 1}})),
                        )
                        .await;
                    if let Err(e) = result {
                        tracing::error!("{e:?}")
                    }
                }
                _ => {
                    tracing::error!("backup job {job_name} failed");
                    let result = deployments
                        .patch(
                            &format!("minecraft-{server_name}"),
                            &PatchParams::default(),
                            &Patch::Merge(&json!({"spec": {"replicas": 1}})),
                        )
                        .await;
                    if let Err(e) = result {
                        tracing::error!("{e:?}")
                    }
                }
            }
        }
    });

    Ok(())
}

fn should_op(server_name: &str) -> bool {
    match server_name {
        "creative" => true,
        _ => false,
    }
}
