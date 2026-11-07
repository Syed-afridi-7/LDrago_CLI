use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_lists_tui_command_and_description() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("tui"))
        .stdout(predicate::str::contains("Launch interactive Terminal User Interface (TUI)"));
}

#[test]
fn test_cli_tui_help_displays_usage_and_description() {
    let mut cmd = Command::cargo_bin("ldrago").unwrap();
    cmd.args(["tui", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Launch interactive Terminal User Interface (TUI)"))
        .stdout(predicate::str::contains("Usage:"));
}
