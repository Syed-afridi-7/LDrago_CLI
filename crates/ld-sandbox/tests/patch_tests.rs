use ld_sandbox::patch::PatchEngine;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_patch_engine_computes_diff() {
    let original = "fn main() {\n    println!(\"old\");\n}\n";
    let replacement = "fn main() {\n    println!(\"new\");\n}\n";

    let (diff, added, removed) = PatchEngine::compute_diff(original, replacement, "main.rs");
    assert!(diff.contains("-    println!(\"old\");"));
    assert!(diff.contains("+    println!(\"new\");"));
    assert_eq!(added, 1);
    assert_eq!(removed, 1);
}

#[test]
fn test_patch_engine_dry_run_does_not_modify_file() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("code.rs");
    fs::write(&file, "initial content").expect("write initial");

    let result = PatchEngine::apply(&file, "modified content", true).expect("dry run apply");
    assert!(!result.applied);
    assert_eq!(fs::read_to_string(&file).expect("read"), "initial content");
}

#[test]
fn test_patch_engine_apply_and_rollback() {
    let dir = tempdir().expect("tempdir");
    let file = dir.path().join("code.rs");
    fs::write(&file, "version 1").expect("write v1");

    let result = PatchEngine::apply(&file, "version 2", false).expect("apply v2");
    assert!(result.applied);
    assert_eq!(fs::read_to_string(&file).expect("read"), "version 2");

    let rolled_back = PatchEngine::rollback(&file).expect("rollback");
    assert!(rolled_back);
    assert_eq!(fs::read_to_string(&file).expect("read"), "version 1");
}
