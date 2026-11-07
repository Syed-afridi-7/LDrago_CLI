use anyhow::{Context, Result};
use clap::Subcommand;
use ld_context::{ScanOptions, WorkspaceScanner};
use ld_policy::{AuditEntry, AuditLogger, RiskLevel};
use ld_sandbox::checkpoint::CheckpointManager;
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum CheckpointAction {
    /// Create a new workspace snapshot checkpoint
    Create {
        /// Description of the checkpoint
        #[arg(short, long, default_value = "manual checkpoint")]
        description: String,
    },
    /// List all saved workspace checkpoints
    List,
    /// Restore workspace files from a previous checkpoint
    Restore {
        /// Checkpoint ID to restore (e.g., cp-20261107-123456)
        id: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

pub fn execute_checkpoint(action: CheckpointAction) -> Result<()> {
    let cwd = env::current_dir().context("get current directory")?;
    let manager = CheckpointManager::new(&cwd);
    let audit_file = cwd.join(".ldrago").join("audit.jsonl");
    let audit = AuditLogger::new(audit_file);

    match action {
        CheckpointAction::Create { description } => {
            println!("💾 Creating Workspace Checkpoint...");

            // Use WorkspaceScanner with custom ignores to find files to snapshot
            let mut options = ScanOptions::default();
            options.include_content = false;
            options.custom_ignores.push(".ldrago".to_string());
            options.custom_ignores.push("target".to_string());
            options.custom_ignores.push(".git".to_string());

            let scanner = WorkspaceScanner::new(options);
            let files = scanner.scan(&cwd)?;
            let file_paths: Vec<PathBuf> = files.into_iter().map(|f| f.path).collect();

            let meta = manager.create_checkpoint(&description, &file_paths)?;

            let entry = AuditEntry {
                timestamp: chrono::Utc::now(),
                session_id: "ldrago-checkpoint".to_string(),
                brain: "sandbox".to_string(),
                action: format!("checkpoint_create: {}", meta.id),
                risk: RiskLevel::Low.to_string(),
                allowed: true,
                details: format!("Captured {} files: {}", meta.file_count, description),
            };
            let _ = audit.log_sync(&entry);

            println!("✓ Checkpoint created successfully!");
            println!("  ID         : {}", meta.id);
            println!("  Timestamp  : {}", meta.timestamp.to_rfc3339());
            println!("  Description: {}", meta.description);
            println!("  Files      : {} snapshot files stored in .ldrago/checkpoints/{}", meta.file_count, meta.id);
        }

        CheckpointAction::List => {
            let checkpoints = manager.list_checkpoints()?;
            if checkpoints.is_empty() {
                println!("No workspace checkpoints found in .ldrago/checkpoints/");
                println!("Run 'ldrago checkpoint create' to capture a snapshot.");
                return Ok(());
            }

            println!("================================================================================");
            println!("                    LDRAGO WORKSPACE CHECKPOINTS                                ");
            println!("================================================================================");
            println!("{:<24} {:<24} {:<8} {:<20}", "CHECKPOINT ID", "TIMESTAMP", "FILES", "DESCRIPTION");
            println!("--------------------------------------------------------------------------------");
            for cp in checkpoints {
                let ts = cp.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                println!("{:<24} {:<24} {:<8} {:<20}", cp.id, ts, cp.file_count, cp.description);
            }
            println!("--------------------------------------------------------------------------------");
            println!("To restore: 'ldrago checkpoint restore <CHECKPOINT_ID>'");
        }

        CheckpointAction::Restore { id, yes } => {
            if !yes {
                println!("⚠️ WARNING: Restoring a checkpoint will overwrite existing files in workspace.");
                print!("Proceed with restoring checkpoint '{}'? [y/N]: ", id);
                io::stdout().flush().context("flush stdout")?;
                let mut input = String::new();
                io::stdin().read_line(&mut input).context("read user input")?;
                let trimmed = input.trim().to_ascii_lowercase();
                if trimmed != "y" && trimmed != "yes" {
                    println!("Checkpoint restoration aborted by user.");
                    return Ok(());
                }
            }

            let restored_count = manager.restore_checkpoint(&id)?;

            let entry = AuditEntry {
                timestamp: chrono::Utc::now(),
                session_id: "ldrago-checkpoint".to_string(),
                brain: "sandbox".to_string(),
                action: format!("checkpoint_restore: {}", id),
                risk: RiskLevel::High.to_string(),
                allowed: true,
                details: format!("Restored {} files from checkpoint {}", restored_count, id),
            };
            let _ = audit.log_sync(&entry);

            println!("✓ Checkpoint '{}' restored successfully! ({} files overwritten/reverted)", id, restored_count);
        }
    }

    Ok(())
}
