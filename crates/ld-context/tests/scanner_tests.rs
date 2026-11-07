use ld_context::scanner::{WorkspaceScanner, ScanOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_workspace_scanner_respects_ignores_and_filters_binaries() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // Create folder structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("README.md"), "# Test Project\n").unwrap();
    fs::write(root.join(".git/config"), "secret git internals\n").unwrap();
    fs::write(root.join("target/debug.bin"), vec![0u8, 159, 255, 0]).unwrap();
    fs::write(root.join(".gitignore"), "ignored.txt\n*.tmp\n").unwrap();
    fs::write(root.join("ignored.txt"), "should not be scanned\n").unwrap();
    fs::write(root.join("cache.tmp"), "temporary cache file\n").unwrap();

    let scanner = WorkspaceScanner::new(ScanOptions::default());
    let files = scanner.scan(root).expect("scan succeeds");

    let relative_paths: Vec<String> = files.iter().map(|f| f.relative_path.to_string_lossy().replace('\\', "/")).collect();
    println!("relative_paths: {:?}", relative_paths);
    
    assert!(relative_paths.contains(&"src/main.rs".to_string()));
    assert!(relative_paths.contains(&"README.md".to_string()));
    assert!(!relative_paths.iter().any(|p| p == ".git" || p.starts_with(".git/")));
    assert!(!relative_paths.iter().any(|p| p == "target" || p.starts_with("target/")));
    assert!(!relative_paths.contains(&"ignored.txt".to_string()));
    assert!(!relative_paths.contains(&"cache.tmp".to_string()));
}

#[test]
fn test_workspace_scanner_custom_ignore_and_max_file_size() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("docs/normal.txt"), "hello world\n").unwrap();
    fs::write(root.join("docs/huge.txt"), vec![b'a'; 200_000]).unwrap(); // 200 KB
    fs::write(root.join(".ldragoignore"), "secret_*.txt\n").unwrap();
    fs::write(root.join("secret_keys.txt"), "api_key=12345\n").unwrap();

    let mut options = ScanOptions::default();
    options.max_file_size_bytes = 100_000; // 100 KB max

    let scanner = WorkspaceScanner::new(options);
    let files = scanner.scan(root).expect("scan succeeds");

    let relative_paths: Vec<String> = files.iter().map(|f| f.relative_path.to_string_lossy().replace('\\', "/")).collect();

    assert!(relative_paths.contains(&"docs/normal.txt".to_string()));
    assert!(!relative_paths.contains(&"docs/huge.txt".to_string()));
    assert!(!relative_paths.contains(&"secret_keys.txt".to_string()));
}
