use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub brain: String,
    pub action: String,
    pub risk: String,
    pub allowed: bool,
    pub details: String,
}

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new<P: AsRef<Path>>(log_path: P) -> Self {
        Self {
            log_path: log_path.as_ref().to_path_buf(),
        }
    }

    pub fn log_sync(&self, entry: &AuditEntry) -> Result<()> {
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent).context("create audit parent directory")?;
        }

        let serialized = serde_json::to_string(entry).context("serialize audit entry")?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .context("open audit log file for append")?;

        writeln!(file, "{}", serialized).context("write audit log line")?;
        Ok(())
    }

    pub async fn log(&self, entry: AuditEntry) -> Result<()> {
        let entry_clone = entry.clone();
        let logger = Self {
            log_path: self.log_path.clone(),
        };
        tokio::task::spawn_blocking(move || logger.log_sync(&entry_clone))
            .await
            .context("spawn blocking audit log")?
    }
}
