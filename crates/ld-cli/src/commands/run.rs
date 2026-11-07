use anyhow::{bail, Context, Result};
use ld_policy::{AuditEntry, AuditLogger, PolicyEngine, RiskLevel, SafetyGate, SafetyVerdict};
use ld_sandbox::runner::CommandRunner;
use std::env;
use std::io::{self, Write};
use std::time::Duration;

pub async fn execute_run(command_line: &str, yes: bool, timeout_secs: u64) -> Result<()> {
    let cwd = env::current_dir().context("get current directory")?;
    let audit_file = cwd.join(".ldrago").join("audit.jsonl");
    let audit = AuditLogger::new(audit_file);

    // 1. Safety Gate analysis
    let safety = SafetyGate::default();
    if let SafetyVerdict::Violation { reason, risk } = safety.check_command(command_line) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now(),
            session_id: "ldrago-run".to_string(),
            brain: "sandbox".to_string(),
            action: format!("run: {}", command_line),
            risk: risk.to_string(),
            allowed: false,
            details: format!("Blocked by Safety Gate: {}", reason),
        };
        let _ = audit.log_sync(&entry);

        eprintln!("\n[SECURITY BLOCK] Dangerous Command Intercepted:");
        eprintln!("  Reason: {}", reason);
        eprintln!("  Risk Level: {}", risk);
        bail!("Command blocked by safety policy");
    }

    // 2. Policy Engine authorization
    let policy = PolicyEngine::for_brain("sandbox");
    let permission = policy.evaluate_tool("run_command");

    println!("⚡ LDrago Sandboxed Execution");
    println!("--------------------------------------------------------");
    println!("  Command  : {}", command_line);
    println!("  Directory: {}", cwd.display());
    println!("  Policy   : {}", permission);
    println!("  Timeout  : {}s", timeout_secs);
    println!("--------------------------------------------------------");

    // 3. User approval prompt if not --yes
    if !yes {
        print!("Authorize and execute sandboxed command? [y/N]: ");
        io::stdout().flush().context("flush stdout")?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("read user input")?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed != "y" && trimmed != "yes" {
            println!("Command execution aborted by user.");
            return Ok(());
        }
    }

    // 4. Execute command in scrubbed environment
    let runner = CommandRunner::new(Duration::from_secs(timeout_secs));
    let result = runner.execute(command_line, &cwd).await?;

    let entry = AuditEntry {
        timestamp: chrono::Utc::now(),
        session_id: "ldrago-run".to_string(),
        brain: "sandbox".to_string(),
        action: format!("run: {}", command_line),
        risk: RiskLevel::High.to_string(),
        allowed: true,
        details: format!(
            "exit_code={:?}, success={}, duration={}ms",
            result.exit_code, result.success, result.duration_ms
        ),
    };
    let _ = audit.log_sync(&entry);

    if !result.stdout.is_empty() {
        print!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }

    println!("\n--------------------------------------------------------");
    if result.timed_out {
        eprintln!("❌ Command timed out after {}s", timeout_secs);
        bail!("Process execution timed out");
    } else if result.success {
        println!("✓ Finished successfully ({} ms, exit code 0)", result.duration_ms);
    } else {
        let code = result.exit_code.unwrap_or(-1);
        eprintln!("❌ Command failed with exit code {} ({} ms)", code, result.duration_ms);
        bail!("Command failed with exit code {}", code);
    }

    Ok(())
}
