use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub auth: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Backup {
    pub filename: String,
    pub bytes: u64,
    pub date: String,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Starting,
    Running,
    Stopped,
    Unknown,
}

impl Display for ServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            ServerStatus::Running => "running",
            ServerStatus::Stopped => "stopped",
            ServerStatus::Starting => "starting",
            ServerStatus::Unknown => "unknown",
        };

        write!(f, "{status}")
    }
}

#[derive(Clone, PartialEq)]
pub enum Server {
    Main,
    Creative,
}
