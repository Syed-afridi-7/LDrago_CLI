use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Locate the `training` directory relative to current working directory or binary.
pub fn find_training_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to get current working directory")?;

    // Check direct cwd / training
    if cwd.join("training").join("pyproject.toml").is_file() {
        return Ok(cwd.join("training"));
    }
    // Check if cwd is already the training directory
    if cwd.join("pyproject.toml").is_file() && cwd.join("src").is_dir() {
        return Ok(cwd);
    }
    // Check ancestors
    for ancestor in cwd.ancestors() {
        let cand = ancestor.join("training");
        if cand.join("pyproject.toml").is_file() {
            return Ok(cand);
        }
    }

    // Fallback: check relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        for ancestor in exe_path.ancestors() {
            let cand = ancestor.join("training");
            if cand.join("pyproject.toml").is_file() {
                return Ok(cand);
            }
        }
    }

    Ok(cwd.join("training"))
}

/// Construct the training Command for `uv run python -m src.train.train_micro`
pub fn build_train_command(training_dir: &Path, brain: &str, micro: bool, steps: u32) -> Command {
    let mut cmd = Command::new("uv");
    cmd.args(["run", "python", "-m", "src.train.train_micro"]);

    // Command arguments
    cmd.arg("--brain").arg(brain);
    cmd.arg("--steps").arg(steps.to_string());
    if micro {
        cmd.arg("--micro");
    }

    cmd.current_dir(training_dir);

    // Preserve or set UV_CACHE_DIR to repo cache if available
    if std::env::var("UV_CACHE_DIR").is_err() {
        if let Some(parent) = training_dir.parent() {
            let cache_dir = parent.join(".uv_cache");
            if cache_dir.exists() {
                cmd.env("UV_CACHE_DIR", cache_dir);
            }
        }
    }

    cmd
}

/// Execute training script and stream stdout/stderr to user.
pub fn execute_train(brain: &str, micro: bool, steps: u32) -> Result<()> {
    let training_dir = find_training_dir()?;
    println!(">> LDrago Training Harness");
    println!("   Target Brain : {}", brain);
    println!("   Micro Mode   : {}", if micro { "enabled" } else { "disabled" });
    println!("   Steps        : {}", steps);
    println!("   Working Dir  : {}", training_dir.display());
    println!("--------------------------------------------------------");

    let mut cmd = build_train_command(&training_dir, brain, micro, steps);

    // Stream child process output directly
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn().with_context(|| {
        format!(
            "Failed to execute training process with 'uv' in '{}'. Ensure 'uv' is installed and in PATH.",
            training_dir.display()
        )
    })?;

    let status = child.wait().context("Failed waiting for training process")?;

    if status.success() {
        println!("--------------------------------------------------------");
        println!(">> Training completed successfully for [{}] ({} steps).", brain, steps);
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Training process exited with error status: {:?}",
            status.code()
        ))
    }
}
