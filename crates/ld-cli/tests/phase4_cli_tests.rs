use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_sandboxed_run_success() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.args(["run", "echo sandboxed_run_success", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sandboxed_run_success"))
        .stdout(predicate::str::contains("Finished successfully"));
}

#[test]
fn test_cli_sandboxed_run_security_blocks_destructive() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.args(["run", "rm -rf /", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SECURITY BLOCK"))
        .stderr(predicate::str::contains("Destructive command pattern blocked"));
}

#[test]
fn test_cli_patch_dry_run_and_apply_and_rollback() {
    let dir = tempdir().expect("tempdir");
    let test_file = dir.path().join("code.txt");
    fs::write(&test_file, "initial version line 1\n").expect("write initial");

    // 1. Dry run
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.current_dir(dir.path())
        .args(["patch", "code.txt", "--content", "modified version line 1\n", "--dry-run", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN (Preview Only)"));

    // File should remain unchanged
    assert_eq!(fs::read_to_string(&test_file).expect("read"), "initial version line 1\n");

    // 2. Live patch
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.current_dir(dir.path())
        .args(["patch", "code.txt", "--content", "modified version line 1\n", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully applied patch"));

    assert_eq!(fs::read_to_string(&test_file).expect("read"), "modified version line 1\n");

    // 3. Rollback
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.current_dir(dir.path())
        .args(["rollback", "code.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully rolled back"));

    assert_eq!(fs::read_to_string(&test_file).expect("read"), "initial version line 1\n");
}

#[test]
fn test_cli_checkpoint_create_and_list() {
    let dir = tempdir().expect("tempdir");
    let dummy = dir.path().join("dummy.txt");
    fs::write(&dummy, "content for checkpoint test").expect("write dummy");

    // Create checkpoint
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.current_dir(dir.path())
        .args(["checkpoint", "create", "-d", "test snapshot"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Checkpoint created successfully"))
        .stdout(predicate::str::contains("test snapshot"));

    // List checkpoint
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.current_dir(dir.path())
        .args(["checkpoint", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("LDRAGO WORKSPACE CHECKPOINTS"))
        .stdout(predicate::str::contains("test snapshot"));
}
