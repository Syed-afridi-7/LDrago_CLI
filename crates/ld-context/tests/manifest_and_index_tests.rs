use ld_context::index::KeywordIndex;
use ld_context::manifest::ProjectManifest;
use ld_context::scanner::{ScanOptions, WorkspaceScanner};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_manifest_generation_and_keyword_indexing() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "pub struct TransformerBrain {\n    dim: usize,\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/config.py"),
        "def load_config():\n    return {'dim': 32}\n",
    )
    .unwrap();

    let scanner = WorkspaceScanner::new(ScanOptions::default());
    let files = scanner.scan(root).unwrap();

    let manifest = ProjectManifest::build(root, &files);
    assert_eq!(manifest.total_files, 2);
    assert!(manifest.languages.contains_key("Rust"));
    assert!(manifest.languages.contains_key("Python"));
    assert!(manifest.ascii_tree.contains("src"));

    let index = KeywordIndex::build(&files);
    let results = index.search("TransformerBrain", 5);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].line_number, 1);
    assert!(results[0]
        .line_content
        .contains("pub struct TransformerBrain"));
    assert!(results[0].context_excerpt.contains("dim: usize"));
}
