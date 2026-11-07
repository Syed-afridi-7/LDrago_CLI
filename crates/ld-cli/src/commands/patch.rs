use anyhow::{bail, Context, Result};
use ld_policy::{AuditEntry, AuditLogger, RiskLevel};
use ld_sandbox::containment::PathContainment;
use ld_sandbox::patch::PatchEngine;
use std::env;
use std::io::{self, Write};

pub fn execute_patch(file: &str, content: &str, dry_run: bool, yes: bool) -> Result<()> {
    let cwd = env::current_dir().context("get current directory")?;
    let audit_file = cwd.join(".ldrago").join("audit.jsonl");
    let audit = AuditLogger::new(audit_file);

    // 1. Validate boundary containment
    let containment = PathContainment::new(&cwd);
    let target_path = match containment.resolve_in_bounds(file) {
        Ok(p) => p,
        Err(e) => {
            let entry = AuditEntry {
                timestamp: chrono::Utc::now(),
                session_id: "ldrago-patch".to_string(),
                brain: "sandbox".to_string(),
                action: format!("patch: {}", file),
                risk: RiskLevel::Dangerous.to_string(),
                allowed: false,
                details: format!("Containment violation: {}", e),
            };
            let _ = audit.log_sync(&entry);
            bail!("Path containment violation: {}", e);
        }
    };

    let original = if target_path.exists() {
        std::fs::read_to_string(&target_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 2. Compute unified diff
    let (diff_text, added, removed) = PatchEngine::compute_diff(&original, content, file);

    println!("📝 LDrago Atomic Patch Inspection");
    println!("--------------------------------------------------------");
    println!("  Target  : {}", target_path.display());
    println!("  Changes : +{} / -{} lines", added, removed);
    println!("  Mode    : {}", if dry_run { "DRY RUN (Preview Only)" } else { "LIVE ATOMIC MUTATION" });
    println!("--------------------------------------------------------");
    println!("{}", diff_text);
    println!("--------------------------------------------------------");

    if dry_run {
        println!("[DRY RUN] Patch verified. No files were modified.");
        return Ok(());
    }

    // 3. User confirmation if not --yes
    if !yes {
        print!("Apply this atomic patch to '{}'? [y/N]: ", file);
        io::stdout().flush().context("flush stdout")?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("read user input")?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("Patch application aborted by user.");
            return Ok(());
        }
    }

    // 4. Apply patch with automatic .bak backup
    let result = PatchEngine::apply(&target_path, content, false)?;

    let entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        session_id: "ldrago-patch".to_string(),
        brain: "sandbox".to_string(),
        action: format!("patch: {}", file),
        risk: RiskLevel::Medium.to_string(),
        allowed: true,
        details: format!("+{} / -{} lines applied to {}", result.lines_added, result.lines_removed, file),
    };
    let _ = audit.log_sync(&entry);

    println!("✓ Successfully applied patch to '{}' (+{} / -{} lines).", file, result.lines_added, result.lines_removed);
    println!("  Backup created: '{}.bak'", target_path.display());

    Ok(())
}

pub fn execute_rollback(file: &str) -> Result<()> {
    let cwd = env::current_dir().context("get current directory")?;
    let containment = PathContainment::new(&cwd);
    let target_path = containment.resolve_in_bounds(file)?;

    if PatchEngine::rollback(&target_path)? {
        println!("✓ Successfully rolled back '{}' from backup ('{}.bak').", file, target_path.display());
    } else {
        println!("❌ No backup found for '{}' (expected '{}.bak').", file, target_path.display());
    }

    Ok(())
}
