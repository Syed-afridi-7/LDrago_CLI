use crate::agent::telemetry::log_safety_event;
use crate::agent::tool_parser::ToolInvocation;
use anyhow::Result;
use ld_policy::{SafetyGate, SafetyVerdict};
use ld_sandbox::{CommandRunner, PathContainment};
use std::path::PathBuf;

/// Safe dispatcher that bridges parsed LLM tool invocations with isolated execution.
///
/// Ensures strict workspace path containment, enforces security policies against destructive
/// operations via `SafetyGate`, and logs safety and execution events without polluting stdout.
pub struct ToolDispatcher {
    pub workspace_root: PathBuf,
    pub safety_gate: SafetyGate,
    pub command_runner: CommandRunner,
    pub containment: PathContainment,
}

impl ToolDispatcher {
    /// Initialize a new tool dispatcher bounded to the specified workspace directory.
    pub fn new(workspace_root: PathBuf) -> Self {
        let containment = PathContainment::new(&workspace_root);
        let safety_gate = SafetyGate::default();
        let command_runner = CommandRunner::default();
        Self {
            workspace_root,
            safety_gate,
            command_runner,
            containment,
        }
    }

    /// Execute a single tool invocation within the security boundary.
    pub async fn execute(&self, tool: &ToolInvocation) -> Result<String> {
        match tool {
            ToolInvocation::ReadFile { path } => {
                if path.trim().is_empty() {
                    return Ok("[ERROR] Path cannot be empty.".to_string());
                }

                let resolved = match self.containment.resolve_in_bounds(path) {
                    Ok(p) => p,
                    Err(_) => return Ok("[ERROR] Path escapes workspace containment.".to_string()),
                };

                if !resolved.exists() {
                    return Ok(format!("[ERROR] File '{}' does not exist.", path));
                }
                if resolved.is_dir() {
                    return Ok(format!("[ERROR] Path '{}' is a directory, not a file.", path));
                }

                match tokio::fs::read_to_string(&resolved).await {
                    Ok(content) => Ok(content),
                    Err(err) => Ok(format!("[ERROR] Failed to read file '{}': {}", path, err)),
                }
            }

            ToolInvocation::WriteFile { path, content } => {
                if path.trim().is_empty() {
                    return Ok("[ERROR] Path cannot be empty.".to_string());
                }

                let resolved = match self.containment.resolve_in_bounds(path) {
                    Ok(p) => p,
                    Err(_) => return Ok("[ERROR] Path escapes workspace containment.".to_string()),
                };

                if let Some(parent) = resolved.parent() {
                    if !parent.exists() {
                        if let Err(err) = tokio::fs::create_dir_all(parent).await {
                            return Ok(format!(
                                "[ERROR] Failed to create directories for '{}': {}",
                                path, err
                            ));
                        }
                    }
                }

                match tokio::fs::write(&resolved, content).await {
                    Ok(_) => Ok(format!(
                        "[OK] Successfully wrote {} bytes to '{}'.",
                        content.len(),
                        path
                    )),
                    Err(err) => Ok(format!("[ERROR] Failed to write to '{}': {}", path, err)),
                }
            }

            ToolInvocation::PatchFile {
                path,
                target,
                replacement,
            } => {
                if path.trim().is_empty() {
                    return Ok("[ERROR] Path cannot be empty.".to_string());
                }

                let resolved = match self.containment.resolve_in_bounds(path) {
                    Ok(p) => p,
                    Err(_) => return Ok("[ERROR] Path escapes workspace containment.".to_string()),
                };

                if !resolved.exists() {
                    return Ok(format!("[ERROR] File '{}' does not exist.", path));
                }

                let current = match tokio::fs::read_to_string(&resolved).await {
                    Ok(c) => c,
                    Err(err) => {
                        return Ok(format!(
                            "[ERROR] Failed to read file '{}' for patching: {}",
                            path, err
                        ))
                    }
                };

                if !current.contains(target) {
                    return Ok(format!("[ERROR] Target block not found in '{}'.", path));
                }

                let patched = current.replacen(target, replacement, 1);
                match tokio::fs::write(&resolved, &patched).await {
                    Ok(_) => Ok(format!("[OK] Successfully patched '{}'.", path)),
                    Err(err) => {
                        Ok(format!("[ERROR] Failed to write patched content to '{}': {}", path, err))
                    }
                }
            }

            ToolInvocation::RunCommand { command } => {
                if command.trim().is_empty() {
                    return Ok("[ERROR] Command cannot be empty.".to_string());
                }

                // Zero-trust evaluation via policy safety gate
                let verdict = self.safety_gate.check_command(command);

                // [SAFETY AUDIT HOOK] Log security verdict silently to telemetry buffer
                log_safety_event(&verdict, command);

                match verdict {
                    SafetyVerdict::Violation { reason, risk } => Ok(format!(
                        "[SECURITY BLOCK] Policy Violation: {} (Risk: {})",
                        reason, risk
                    )),
                    SafetyVerdict::Safe => {
                        match self.command_runner.execute(command, &self.workspace_root).await {
                            Ok(result) => {
                                let mut out = String::new();
                                if !result.stdout.is_empty() {
                                    out.push_str(&result.stdout);
                                }
                                if !result.stderr.is_empty() {
                                    if !out.is_empty() && !out.ends_with('\n') {
                                        out.push('\n');
                                    }
                                    out.push_str(&result.stderr);
                                }
                                if out.is_empty() {
                                    if result.success {
                                        out = "[OK] Command executed successfully with no output."
                                            .to_string();
                                    } else {
                                        out = format!(
                                            "[ERROR] Command exited with code {:?}",
                                            result.exit_code
                                        );
                                    }
                                }
                                Ok(out)
                            }
                            Err(err) => Ok(format!("[ERROR] Command execution failed: {}", err)),
                        }
                    }
                }
            }

            ToolInvocation::ListFiles { path } => {
                let target_dir = match path {
                    None => self.workspace_root.clone(),
                    Some(p) if p.trim().is_empty() => {
                        return Ok("[ERROR] Path cannot be empty.".to_string());
                    }
                    Some(p) => match self.containment.resolve_in_bounds(p) {
                        Ok(resolved) => resolved,
                        Err(_) => {
                            return Ok("[ERROR] Path escapes workspace containment.".to_string())
                        }
                    },
                };

                if !target_dir.exists() {
                    return Ok(format!(
                        "[ERROR] Directory does not exist: {}",
                        target_dir.display()
                    ));
                }
                if !target_dir.is_dir() {
                    return Ok(format!(
                        "[ERROR] Path is not a directory: {}",
                        target_dir.display()
                    ));
                }

                let mut entries = Vec::new();
                let mut dir = tokio::fs::read_dir(&target_dir).await?;
                while let Some(entry) = dir.next_entry().await? {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry
                        .file_type()
                        .await
                        .map(|t| t.is_dir())
                        .unwrap_or(false);
                    if is_dir {
                        entries.push(format!("{}/", name));
                    } else {
                        entries.push(name);
                    }
                }
                entries.sort();
                Ok(entries.join("\n"))
            }
        }
    }
}
