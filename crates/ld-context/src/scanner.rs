use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub max_file_size_bytes: u64,
    pub include_content: bool,
    pub custom_ignores: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 1_048_576, // 1 MB default
            include_content: true,
            custom_ignores: vec![
                ".git".to_string(),
                "target".to_string(),
                ".venv".to_string(),
                "__pycache__".to_string(),
                "node_modules".to_string(),
                ".uv_cache".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub size_bytes: u64,
    pub is_binary: bool,
    pub content: Option<String>,
}

pub struct WorkspaceScanner {
    options: ScanOptions,
}

impl WorkspaceScanner {
    pub fn new(options: ScanOptions) -> Self {
        Self { options }
    }

    pub fn scan<P: AsRef<Path>>(&self, root: P) -> Result<Vec<ScannedFile>> {
        let root = root.as_ref();
        let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let mut builder = WalkBuilder::new(root);
        builder.hidden(false);
        builder.git_ignore(true);
        builder.git_global(true);
        builder.git_exclude(true);
        builder.add_custom_ignore_filename(".ldragoignore");

        let mut scanned_files = Vec::new();

        for result in builder.build() {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let relative_path = match canonical_path.strip_prefix(&canonical_root) {
                Ok(rel) => rel.to_path_buf(),
                Err(_) => match path.strip_prefix(root) {
                    Ok(rel) => rel.to_path_buf(),
                    Err(_) => path.to_path_buf(),
                },
            };

            // Filter out any component matching custom_ignores
            let has_ignored_component = relative_path.components().any(|comp| {
                let s = comp.as_os_str().to_string_lossy();
                self.options.custom_ignores.iter().any(|ig| ig == &s)
            });
            if has_ignored_component {
                continue;
            }

            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = metadata.len();
            if size_bytes > self.options.max_file_size_bytes {
                continue;
            }

            // Binary heuristic: check first 1024 bytes for null byte
            let is_bin = match is_binary_file(path) {
                Ok(b) => b,
                Err(_) => true,
            };

            if is_bin {
                continue; // Skip binaries from context extraction
            }

            let content = if self.options.include_content {
                match fs::read_to_string(path) {
                    Ok(text) => Some(text),
                    Err(_) => None,
                }
            } else {
                None
            };

            scanned_files.push(ScannedFile {
                path: path.to_path_buf(),
                relative_path,
                size_bytes,
                is_binary: false,
                content,
            });
        }

        // Sort files by relative path for deterministic ordering
        scanned_files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        Ok(scanned_files)
    }
}

fn is_binary_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    use std::io::Read;
    let mut file = fs::File::open(path).context("open file for binary check")?;
    let mut buffer = [0u8; 1024];
    let bytes_read = file.read(&mut buffer).context("read buffer")?;
    Ok(buffer[..bytes_read].contains(&0))
}
