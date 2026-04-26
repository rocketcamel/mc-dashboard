mod backup;

use askama::Template;

pub use backup::backup_world;

#[derive(Template)]
#[template(path = "restore-job.yaml", escape = "none")]
pub struct BackupJob<'a> {
    pub server_name: &'a str,
    pub backup_file_name: &'a str,
    pub should_op: bool,
}
