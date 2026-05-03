mod backup;

use openssh::Session;
use serde::Serialize;

pub struct Storage {
    pub ssh: Session,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Backup {
    pub filename: String,
    pub bytes: u64,
    pub date: String,
}
