use anyhow::{bail, Result};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PathContainment {
    canonical_root: PathBuf,
}

impl PathContainment {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let p = root.as_ref();
        let canonical_root = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        Self { canonical_root }
    }

    pub fn resolve_in_bounds<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let requested = path.as_ref();

        // 1. Check for suspicious traversal components
        let mut depth: isize = 0;
        for comp in requested.components() {
            match comp {
                Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        bail!("Security Violation: Path escapes workspace root via '..'");
                    }
                }
                Component::Normal(_) => {
                    depth += 1;
                }
                Component::RootDir | Component::Prefix(_) => {
                    // Absolute path: check directly if within canonical_root
                    let req_buf = requested.to_path_buf();
                    let can = req_buf.canonicalize().unwrap_or(req_buf);
                    if !can.starts_with(&self.canonical_root) {
                        bail!("Security Violation: Absolute path is outside workspace root");
                    }
                    return Ok(can);
                }
                _ => {}
            }
        }

        // 2. Resolve relative to canonical_root
        let target = self.canonical_root.join(requested);

        // If target exists, canonicalize and verify boundary
        if target.exists() {
            let canonical_target = target.canonicalize()?;
            if !canonical_target.starts_with(&self.canonical_root) {
                bail!("Security Violation: Symlink or path escapes workspace root");
            }
            return Ok(canonical_target);
        }

        // If target doesn't exist yet, normalize cleanly
        let mut normalized = self.canonical_root.clone();
        for comp in requested.components() {
            match comp {
                Component::Normal(c) => normalized.push(c),
                Component::CurDir => {}
                _ => {}
            }
        }

        Ok(normalized)
    }

    pub fn root(&self) -> &Path {
        &self.canonical_root
    }
}
