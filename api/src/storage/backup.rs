use chrono::NaiveDateTime;

use super::{Backup, Storage};
use crate::error::Result;

impl Storage {
    pub async fn get_backups(&self) -> Result<Vec<Backup>> {
        let output = self
            .ssh
            .command("ls")
            .args(["-1s", "--block-size=1", "/backup/minecraft/"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::Error::ssh_command(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut backups: Vec<Backup> = stdout.lines().filter_map(parse_backup).collect();
        backups.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(backups)
    }
}

fn parse_backup(line: &str) -> Option<Backup> {
    let (size_str, filename) = line.trim().split_once(' ')?;
    let name = filename.trim();

    if !name.starts_with("minecraft-main-") {
        return None;
    }

    let bytes = size_str.parse::<u64>().ok()?;
    let date_part = name.strip_prefix("minecraft-main-")?.split('.').next()?;
    let dt = NaiveDateTime::parse_from_str(date_part, "%Y%m%d-%H%M%S").ok()?;

    Some(Backup {
        filename: name.to_string(),
        bytes,
        date: dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_backups_from_ls_output() {
        let ls_output = "\
1904701440 minecraft-main-20260418-212404.tar.gz
1904701440 minecraft-main-20260418-214014.tar.gz
1904697344 minecraft-main-20260418-215625.tar.gz
1904697344 minecraft-main-20260418-221236.tar.gz
1904701440 minecraft-main-20260418-222846.tar.gz
1904697344 minecraft-main-20260418-224457.tar.gz
1904693248 minecraft-main-20260418-230107.tar.gz
1904693248 minecraft-main-20260418-231718.tar.gz
1904701440 minecraft-main-20260418-233329.tar.gz
1904701440 minecraft-main-20260418-234940.tar.gz
1904701440 minecraft-main-20260419-000550.tar.gz
1904697344 minecraft-main-20260419-002201.tar.gz
1904701440 minecraft-main-20260419-003812.tar.gz
1904697344 minecraft-main-20260419-005423.tar.gz";

        let mut backups: Vec<Backup> = ls_output.lines().filter_map(parse_backup).collect();
        backups.sort_by(|a, b| b.date.cmp(&a.date));

        assert_eq!(backups[0].filename, "minecraft-main-20260419-005423.tar.gz");
        assert_eq!(backups[0].date, "2026-04-19T00:54:23");
        assert_eq!(
            backups[13].filename,
            "minecraft-main-20260418-212404.tar.gz"
        );
        assert_eq!(backups[13].date, "2026-04-18T21:24:04");
    }

    #[test]
    fn skips_non_backup_lines() {
        assert!(parse_backup("4096 manual").is_none());
        assert!(parse_backup("0 .mc-backup-lock").is_none());
        assert!(parse_backup("total 37084376").is_none());
    }
}
