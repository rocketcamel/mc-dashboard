use serde::Deserialize;
use strum::{AsRefStr, Display};

pub mod auth;
pub mod backup_world;
pub mod get_backups;
pub mod logs;
pub mod status;
pub mod sync_world;

#[derive(Deserialize, Display, AsRefStr, Clone)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum World {
    Main,
    Creative,
}
