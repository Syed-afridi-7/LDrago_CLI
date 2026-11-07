use ld_sandbox::runner::CommandRunner;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_command_runner_executes_simple_command() {
    let runner = CommandRunner::new(Duration::from_secs(5));
    let dir = tempdir().expect("tempdir");

    #[cfg(target_os = "windows")]
    let cmd = "echo hello sandbox";
    #[cfg(not(target_os = "windows"))]
    let cmd = "echo 'hello sandbox'";

    let result = runner.execute(cmd, dir.path()).await.expect("execute");
    assert!(result.success);
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("hello sandbox"));
    assert!(!result.timed_out);
}

#[tokio::test]
async fn test_command_runner_sanitizes_environment_variables() {
    // Inject a sensitive environment variable into current process
    std::env::set_var("AWS_ACCESS_KEY_SECRET", "super_secret_val_123");
    std::env::set_var("SAFE_CONFIG_VAR", "safe_val_456");

    let clean_env = CommandRunner::sanitize_environment();

    // Sensitive should NOT be present
    assert!(!clean_env.contains_key("AWS_ACCESS_KEY_SECRET"));

    // Safe variable SHOULD be present
    assert_eq!(clean_env.get("SAFE_CONFIG_VAR").map(|s| s.as_str()), Some("safe_val_456"));

    // Clean up
    std::env::remove_var("AWS_ACCESS_KEY_SECRET");
    std::env::remove_var("SAFE_CONFIG_VAR");
}

#[tokio::test]
async fn test_command_runner_times_out_on_hanging_process() {
    let runner = CommandRunner::new(Duration::from_millis(500));
    let dir = tempdir().expect("tempdir");

    #[cfg(target_os = "windows")]
    let cmd = "ping -n 4 127.0.0.1 >nul";
    #[cfg(not(target_os = "windows"))]
    let cmd = "sleep 3";

    let result = runner.execute(cmd, dir.path()).await.expect("execute");
    assert!(!result.success);
    assert!(result.timed_out);
    assert!(result.stderr.contains("timed out"));
}
