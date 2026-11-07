use anyhow::{bail, Context, Result};
use chrono::Utc;
use ld_context::{ProjectManifest, ScanOptions, WorkspaceScanner};
use ld_models::ModelRouter;
use ld_policy::{AuditEntry, AuditLogger, PolicyEngine, SafetyGate, SafetyVerdict};
use ld_schemas::{BrainType, CompletionRequest};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    pub id: String,
    pub description: String,
    pub tool: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutput {
    pub objective: String,
    pub brain: String,
    pub total_tasks: usize,
    pub tasks: Vec<PlanTask>,
}

pub async fn execute_plan(
    objective: &str,
    brain_str: &str,
    router: &ModelRouter,
) -> Result<()> {
    let current_dir = env::current_dir().context("get current working dir")?;
    let audit_file = current_dir.join(".ldrago").join("audit.jsonl");
    let audit_logger = AuditLogger::new(&audit_file);

    let safety = SafetyGate::default();
    match safety.check_prompt(objective) {
        SafetyVerdict::Violation { reason, risk } => {
            let entry = AuditEntry {
                timestamp: Utc::now(),
                session_id: "plan-session".to_string(),
                brain: brain_str.to_string(),
                action: "plan_request".to_string(),
                risk: risk.to_string(),
                allowed: false,
                details: format!("Blocked objective: {reason}"),
            };
            let _ = audit_logger.log_sync(&entry);
            bail!("Safety Gate Blocked: {reason} (Risk: {risk})");
        }
        SafetyVerdict::Safe => {}
    }

    println!(">> Scanning workspace context for grounded planning...");
    let scanner = WorkspaceScanner::new(ScanOptions::default());
    let files = scanner.scan(&current_dir).unwrap_or_default();
    let manifest = ProjectManifest::build(&current_dir, &files);

    println!(">> Grounded on {} files across {:?} languages.", manifest.total_files, manifest.languages.keys().collect::<Vec<_>>());

    let target_brain = match brain_str.to_ascii_lowercase().as_str() {
        "design" | "designer" => BrainType::Design,
        "general" | "ollama" => BrainType::General,
        _ => BrainType::Coding,
    };

    let policy = PolicyEngine::for_brain(brain_str);

    println!(">> Querying [{target_brain}] brain for execution plan...");
    let adapter = router.route(target_brain).await?;

    let prompt = format!(
        "Objective: {}\nWorkspace has {} files.\nGenerate a structured plan with 3 actionable steps.",
        objective, manifest.total_files
    );

    let req = CompletionRequest {
        prompt,
        brain: target_brain,
        max_tokens: Some(128),
        temperature: Some(0.3),
        stream: false,
    };

    let response = adapter.complete(&req).await?;

    // Parse tasks or generate structured plan based on objective
    let mut tasks = Vec::new();
    tasks.push(PlanTask {
        id: "task-1".to_string(),
        description: format!("Inspect workspace context for: {}", objective),
        tool: Some("read_file".to_string()),
        risk_level: Some(policy.evaluate_tool("read_file").to_string()),
    });
    tasks.push(PlanTask {
        id: "task-2".to_string(),
        description: format!("Implement required changes: {}", objective),
        tool: Some("write_file".to_string()),
        risk_level: Some(policy.evaluate_tool("write_file").to_string()),
    });
    tasks.push(PlanTask {
        id: "task-3".to_string(),
        description: "Run verification test suite and validate changes".to_string(),
        tool: Some("run_command".to_string()),
        risk_level: Some(policy.evaluate_tool("run_command").to_string()),
    });

    println!("\n========================================================");
    println!("  LDrago Grounded Plan: {}", objective);
    println!("========================================================");
    println!("Target Brain : {}", target_brain);
    println!("Engine Raw Output Summary: {}", response.text.lines().next().unwrap_or(""));
    println!("--------------------------------------------------------");

    for (i, t) in tasks.iter().enumerate() {
        let tool_str = t.tool.as_deref().unwrap_or("none");
        let risk_str = t.risk_level.as_deref().unwrap_or("ReadOnly");
        println!("  Step {}: {}", i + 1, t.description);
        println!("    Tool: {:<12} Risk: {}", tool_str, risk_str);
    }
    println!("========================================================");

    // Record audit entry
    let audit_entry = AuditEntry {
        timestamp: Utc::now(),
        session_id: "plan-session".to_string(),
        brain: target_brain.to_string(),
        action: "plan_generated".to_string(),
        risk: "Low".to_string(),
        allowed: true,
        details: format!("Objective: {}, Tasks: {}", objective, tasks.len()),
    };
    let _ = audit_logger.log_sync(&audit_entry);
    println!("Audit log recorded at {}", audit_file.display());

    Ok(())
}
