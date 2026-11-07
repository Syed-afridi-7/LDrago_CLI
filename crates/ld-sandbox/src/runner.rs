use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

#[derive(Debug, Clone)]
pub struct CommandRunner {
    pub timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_output_bytes: 1_048_576, // 1 MB
        }
    }
}

impl CommandRunner {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }

    pub fn sanitize_environment() -> HashMap<String, String> {
        let mut clean_env = HashMap::new();
        let sensitive_substrings = [
            "SECRET",
            "TOKEN",
            "PASSWORD",
            "CREDENTIAL",
            "PRIVATE",
            "AWS_ACCESS",
            "OPENAI",
            "ANTHROPIC",
            "BEARER",
        ];

        for (k, v) in env::vars() {
            let k_upper = k.to_uppercase();
            let is_sensitive = sensitive_substrings.iter().any(|s| k_upper.contains(s));
            if !is_sensitive {
                clean_env.insert(k, v);
            }
        }

        clean_env
    }

    pub async fn execute<P: AsRef<Path>>(&self, command_line: &str, cwd: P) -> Result<ExecutionResult> {
        let clean_env = Self::sanitize_environment();
        let start = Instant::now();

        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = Command::new("cmd.exe");
            c.arg("/C").arg(command_line);
            c
        };

        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command_line);
            c
        };

        cmd.current_dir(cwd.as_ref());
        cmd.env_clear();
        cmd.envs(&clean_env);

        let child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("spawn sandboxed process")?;

        match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let stdout_bytes = &output.stdout[..output.stdout.len().min(self.max_output_bytes)];
                let stderr_bytes = &output.stderr[..output.stderr.len().min(self.max_output_bytes)];

                let stdout = String::from_utf8_lossy(stdout_bytes).to_string();
                let stderr = String::from_utf8_lossy(stderr_bytes).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();

                Ok(ExecutionResult {
                    exit_code,
                    success,
                    stdout,
                    stderr,
                    duration_ms,
                    timed_out: false,
                })
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                Ok(ExecutionResult {
                    exit_code: None,
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Process execution timed out after {} seconds", self.timeout.as_secs()),
                    duration_ms,
                    timed_out: true,
                })
            }
        }
    }
}
