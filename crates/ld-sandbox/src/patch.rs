use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResult {
    pub file_path: PathBuf,
    pub diff: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub applied: bool,
}

pub struct PatchEngine;

impl PatchEngine {
    pub fn compute_diff(original: &str, replacement: &str, file_label: &str) -> (String, usize, usize) {
        let diff = TextDiff::from_lines(original, replacement);
        let mut unified = String::new();
        let mut added = 0;
        let mut removed = 0;

        unified.push_str(&format!("--- a/{}\n+++ b/{}\n", file_label, file_label));

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                unified.push_str("\n");
            }
            for op in group {
                for change in diff.iter_changes(op) {
                    match change.tag() {
                        ChangeTag::Insert => {
                            added += 1;
                            unified.push_str(&format!("+{}", change));
                        }
                        ChangeTag::Delete => {
                            removed += 1;
                            unified.push_str(&format!("-{}", change));
                        }
                        ChangeTag::Equal => {
                            unified.push_str(&format!(" {}", change));
                        }
                    }
                }
            }
        }

        (unified, added, removed)
    }

    pub fn apply<P: AsRef<Path>>(target_path: P, new_content: &str, dry_run: bool) -> Result<PatchResult> {
        let target = target_path.as_ref();
        let original = if target.exists() {
            fs::read_to_string(target).context("read target file for diff")?
        } else {
            String::new()
        };

        let file_label = target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let (diff_text, lines_added, lines_removed) = Self::compute_diff(&original, new_content, file_label);

        if !dry_run {
            // Create backup file .bak if original exists
            if target.exists() {
                let mut bak_path = target.to_path_buf();
                bak_path.set_extension(format!(
                    "{}.bak",
                    target.extension().and_then(|s| s.to_str()).unwrap_or("")
                ));
                let _ = fs::copy(target, bak_path);
            }

            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).context("create target parent directory")?;
            }

            fs::write(target, new_content).context("write patched file contents")?;
        }

        Ok(PatchResult {
            file_path: target.to_path_buf(),
            diff: diff_text,
            lines_added,
            lines_removed,
            applied: !dry_run,
        })
    }

    pub fn rollback<P: AsRef<Path>>(target_path: P) -> Result<bool> {
        let target = target_path.as_ref();
        let mut bak_path = target.to_path_buf();
        bak_path.set_extension(format!(
            "{}.bak",
            target.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        if bak_path.exists() {
            fs::copy(&bak_path, target).context("restore backup to target")?;
            fs::remove_file(&bak_path).context("remove backup file")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
