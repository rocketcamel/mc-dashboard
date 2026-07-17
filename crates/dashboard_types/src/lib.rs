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

#[derive(Serialize, Deserialize)]
pub struct BackupRequest {
    pub server_name: World,
    pub backup_file_name: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum World {
    Main,
    Creative,
}

impl AsRef<str> for World {
    fn as_ref(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Creative => "creative",
        }
    }
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main => write!(f, "main"),
            Self::Creative => write!(f, "creative"),
        }
    }
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
