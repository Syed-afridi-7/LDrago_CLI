use clap::Parser;
use ld_cli::commands::train::{build_train_command, find_training_dir};
use ld_cli::{Cli, Commands};
use std::process::Command;

#[test]
fn test_train_cli_argument_parsing_defaults() {
    let cli = Cli::try_parse_from(["ldrago", "train", "coding"]).expect("Failed to parse train command");

    match cli.command {
        Some(Commands::Train { brain, micro, steps }) => {
            assert_eq!(brain, "coding");
            assert_eq!(micro, false);
            assert_eq!(steps, 10);
        }
        other => panic!("Expected Commands::Train, got {:?}", other),
    }
}

#[test]
fn test_train_cli_argument_parsing_custom_args() {
    let cli = Cli::try_parse_from(["ldrago", "train", "design", "--micro", "--steps", "25"])
        .expect("Failed to parse custom train args");

    match cli.command {
        Some(Commands::Train { brain, micro, steps }) => {
            assert_eq!(brain, "design");
            assert_eq!(micro, true);
            assert_eq!(steps, 25);
        }
        other => panic!("Expected Commands::Train, got {:?}", other),
    }
}

#[test]
fn test_train_cli_short_flag_steps() {
    let cli = Cli::try_parse_from(["ldrago", "train", "coding", "-s", "3", "--micro"])
        .expect("Failed to parse short flag steps");

    match cli.command {
        Some(Commands::Train { brain, micro, steps }) => {
            assert_eq!(brain, "coding");
            assert_eq!(micro, true);
            assert_eq!(steps, 3);
        }
        other => panic!("Expected Commands::Train, got {:?}", other),
    }
}

#[test]
fn test_find_training_dir_resolves_valid_dir() {
    let dir = find_training_dir().expect("Failed to resolve training directory");
    assert!(
        dir.join("pyproject.toml").is_file(),
        "Expected pyproject.toml in training dir: {}",
        dir.display()
    );
}

#[test]
fn test_build_train_command_structure() {
    let dir = find_training_dir().expect("Failed to resolve training directory");
    let cmd = build_train_command(&dir, "coding", true, 7);

    let program = cmd.get_program().to_string_lossy();
    assert_eq!(program, "uv");

    let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
    assert_eq!(args, vec![
        "run",
        "python",
        "-m",
        "src.train.train_micro",
        "--brain",
        "coding",
        "--steps",
        "7",
        "--micro",
    ]);

    assert_eq!(cmd.get_current_dir(), Some(dir.as_path()));
}

#[test]
fn test_ldrago_binary_train_help() {
    let exe = env!("CARGO_BIN_EXE_ldrago");
    let output = Command::new(exe)
        .args(["train", "--help"])
        .output()
        .expect("Failed to execute ldrago train --help");

    assert!(output.status.success(), "train --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<BRAIN>"), "Help must include BRAIN argument");
    assert!(stdout.contains("--micro"), "Help must include --micro flag");
    assert!(stdout.contains("--steps"), "Help must include --steps flag");
}
