mod backup;
mod sync;

use askama::Template;

pub use backup::backup_world;
pub use sync::sync_world;

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
