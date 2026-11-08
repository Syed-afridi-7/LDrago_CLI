use ld_cli::agent::dispatcher::ToolDispatcher;
use ld_cli::agent::telemetry::{
    get_safety_events, get_telemetry_stats, log_safety_event, record_inference_telemetry,
};
use ld_cli::agent::tool_parser::ToolInvocation;
use ld_policy::SafetyVerdict;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_dispatcher_read_file_within_workspace() {
    let dir = tempdir().expect("create temp dir");
    let test_file = dir.path().join("sample.txt");
    fs::write(&test_file, "alpha beta gamma 123").expect("write test file");

    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());
    let tool = ToolInvocation::ReadFile {
        path: "sample.txt".to_string(),
    };

    let result = dispatcher.execute(&tool).await.expect("execute read tool");
    assert_eq!(result, "alpha beta gamma 123");
}

#[tokio::test]
async fn test_dispatcher_write_and_read_roundtrip() {
    let dir = tempdir().expect("create temp dir");
    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    let content = "Hello from agent dispatcher roundtrip test!\nLine 2";
    let write_tool = ToolInvocation::WriteFile {
        path: "nested/roundtrip.txt".to_string(),
        content: content.to_string(),
    };

    let write_res = dispatcher.execute(&write_tool).await.expect("write execution");
    assert!(write_res.contains("[OK]"));

    let read_tool = ToolInvocation::ReadFile {
        path: "nested/roundtrip.txt".to_string(),
    };
    let read_res = dispatcher.execute(&read_tool).await.expect("read execution");
    assert_eq!(read_res, content);
}

#[tokio::test]
async fn test_dispatcher_path_containment_blocks_escape() {
    let dir = tempdir().expect("create temp dir");
    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    let escape_tool = ToolInvocation::ReadFile {
        path: "../../etc/passwd".to_string(),
    };
    let res = dispatcher.execute(&escape_tool).await.expect("execute escape tool");
    assert_eq!(res, "[ERROR] Path escapes workspace containment.");

    let escape_write = ToolInvocation::WriteFile {
        path: "../outside.txt".to_string(),
        content: "malicious".to_string(),
    };
    let res_write = dispatcher.execute(&escape_write).await.expect("execute escape write");
    assert_eq!(res_write, "[ERROR] Path escapes workspace containment.");
}

#[tokio::test]
async fn test_dispatcher_empty_path_error() {
    let dir = tempdir().expect("create temp dir");
    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    let empty_read = ToolInvocation::ReadFile {
        path: "   ".to_string(),
    };
    let res_read = dispatcher.execute(&empty_read).await.expect("execute empty read");
    assert!(res_read.starts_with("[ERROR] Path cannot be empty."));

    let empty_write = ToolInvocation::WriteFile {
        path: "".to_string(),
        content: "content".to_string(),
    };
    let res_write = dispatcher.execute(&empty_write).await.expect("execute empty write");
    assert!(res_write.starts_with("[ERROR] Path cannot be empty."));

    let empty_list = ToolInvocation::ListFiles {
        path: Some("".to_string()),
    };
    let res_list = dispatcher.execute(&empty_list).await.expect("execute empty list");
    assert!(res_list.starts_with("[ERROR] Path cannot be empty."));
}

#[tokio::test]
async fn test_dispatcher_safe_command_execution() {
    let dir = tempdir().expect("create temp dir");
    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    let tool = ToolInvocation::RunCommand {
        command: "cargo --version".to_string(),
    };

    let result = dispatcher.execute(&tool).await.expect("execute cargo --version");
    assert!(result.contains("cargo"));
}

#[tokio::test]
async fn test_dispatcher_security_block_on_malicious_command() {
    let dir = tempdir().expect("create temp dir");
    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    // Test format command
    let format_tool = ToolInvocation::RunCommand {
        command: "format C:".to_string(),
    };
    let format_res = dispatcher.execute(&format_tool).await.expect("execute format C:");
    assert!(format_res.starts_with("[SECURITY BLOCK] Policy Violation:"));
    assert!(format_res.contains("Dangerous"));

    // Test rmdir /s /q C:\Windows command
    let rmdir_tool = ToolInvocation::RunCommand {
        command: "rmdir /s /q C:\\Windows".to_string(),
    };
    let rmdir_res = dispatcher.execute(&rmdir_tool).await.expect("execute rmdir Windows");
    assert!(rmdir_res.starts_with("[SECURITY BLOCK] Policy Violation:"));
    assert!(rmdir_res.contains("Dangerous"));
}

#[tokio::test]
async fn test_dispatcher_list_files() {
    let dir = tempdir().expect("create temp dir");
    fs::write(dir.path().join("file_b.txt"), "b").expect("write file b");
    fs::write(dir.path().join("file_a.txt"), "a").expect("write file a");
    fs::create_dir_all(dir.path().join("sub_dir")).expect("create sub_dir");
    fs::write(dir.path().join("sub_dir").join("inner.txt"), "inner").expect("write inner file");

    let dispatcher = ToolDispatcher::new(dir.path().to_path_buf());

    // Root list
    let list_root = ToolInvocation::ListFiles { path: None };
    let root_res = dispatcher.execute(&list_root).await.expect("execute list root");
    assert!(root_res.contains("file_a.txt"));
    assert!(root_res.contains("file_b.txt"));
    assert!(root_res.contains("sub_dir/"));

    // Subdir list
    let list_sub = ToolInvocation::ListFiles {
        path: Some("sub_dir".to_string()),
    };
    let sub_res = dispatcher.execute(&list_sub).await.expect("execute list subdir");
    assert!(sub_res.contains("inner.txt"));
}

#[test]
fn test_silent_telemetry_and_safety_hooks() {
    // Verify inference telemetry hook
    record_inference_telemetry(42, Duration::from_millis(150));
    let stats = get_telemetry_stats();
    assert!(stats.total_tokens >= 42);
    assert!(stats.total_duration_ms >= 150);

    // Verify safety event logging hook
    let verdict = SafetyVerdict::Violation {
        reason: "Test block".to_string(),
        risk: ld_policy::RiskLevel::Dangerous,
    };
    log_safety_event(&verdict, "format C:");
    let events = get_safety_events();
    assert!(events.iter().any(|e| e.command == "format C:"));
}
