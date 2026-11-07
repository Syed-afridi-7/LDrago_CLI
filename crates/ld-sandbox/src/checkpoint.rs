use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub file_count: usize,
    pub files: Vec<String>,
}

pub struct CheckpointManager {
    workspace_root: PathBuf,
    checkpoints_dir: PathBuf,
}

impl CheckpointManager {
    pub fn new<P: AsRef<Path>>(workspace_root: P) -> Self {
        let root = workspace_root.as_ref().to_path_buf();
        let checkpoints_dir = root.join(".ldrago").join("checkpoints");
        Self {
            workspace_root: root,
            checkpoints_dir,
        }
    }

    pub fn create_checkpoint(&self, description: &str, files_to_snapshot: &[PathBuf]) -> Result<CheckpointMetadata> {
        let timestamp = Utc::now();
        let id = format!("cp-{}", timestamp.format("%Y%m%d-%H%M%S"));
        let target_dir = self.checkpoints_dir.join(&id);
        fs::create_dir_all(&target_dir).context("create checkpoint target directory")?;

        let mut stored_files = Vec::new();

        for file_path in files_to_snapshot {
            let relative = if file_path.is_absolute() {
                match file_path.strip_prefix(&self.workspace_root) {
                    Ok(rel) => rel.to_path_buf(),
                    Err(_) => file_path.clone(),
                }
            } else {
                file_path.clone()
            };

            let source = self.workspace_root.join(&relative);
            if source.is_file() {
                let dest = target_dir.join("data").join(&relative);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).context("create checkpoint file parent")?;
                }
                fs::copy(&source, &dest).context("snapshot file into checkpoint")?;
                stored_files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }

        let meta = CheckpointMetadata {
            id: id.clone(),
            timestamp,
            description: description.to_string(),
            file_count: stored_files.len(),
            files: stored_files,
        };

        let meta_file = target_dir.join("metadata.json");
        let serialized = serde_json::to_string_pretty(&meta).context("serialize checkpoint metadata")?;
        fs::write(meta_file, serialized).context("write checkpoint metadata.json")?;

        Ok(meta)
    }

    pub fn list_checkpoints(&self) -> Result<Vec<CheckpointMetadata>> {
        if !self.checkpoints_dir.exists() {
            return Ok(Vec::new());
        }

        let mut checkpoints = Vec::new();
        for entry in fs::read_dir(&self.checkpoints_dir).context("read checkpoints directory")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let meta_file = path.join("metadata.json");
                if meta_file.exists() {
                    if let Ok(content) = fs::read_to_string(&meta_file) {
                        if let Ok(meta) = serde_json::from_str::<CheckpointMetadata>(&content) {
                            checkpoints.push(meta);
                        }
                    }
                }
            }
        }

        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(checkpoints)
    }

    pub fn restore_checkpoint(&self, checkpoint_id: &str) -> Result<usize> {
        let cp_dir = self.checkpoints_dir.join(checkpoint_id);
        if !cp_dir.exists() {
            bail!("Checkpoint '{}' not found", checkpoint_id);
        }

        let data_dir = cp_dir.join("data");
        if !data_dir.exists() {
            bail!("Checkpoint '{}' has no snapshot data", checkpoint_id);
        }

        let mut restored_count = 0;
        let walker = walkdir::WalkDir::new(&data_dir);
        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(&data_dir) {
                    let dest = self.workspace_root.join(rel);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent).context("create restore parent directory")?;
                    }
                    fs::copy(path, &dest).context("restore file to workspace")?;
                    restored_count += 1;
                }
            }
        }

        Ok(restored_count)
    }
}
