use ld_sandbox::containment::PathContainment;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_path_containment_allows_valid_subpaths() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src/utils")).unwrap();
    fs::write(root.join("src/utils/math.rs"), "fn add() {}\n").unwrap();

    let containment = PathContainment::new(root);
    let resolved = containment.resolve_in_bounds("src/utils/math.rs");
    assert!(resolved.is_ok());
    let path = resolved.unwrap();
    assert!(path.ends_with("src/utils/math.rs"));
}

#[test]
fn test_path_containment_blocks_parent_directory_traversal() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    let containment = PathContainment::new(root);
    assert!(containment.resolve_in_bounds("../../etc/shadow").is_err());
    assert!(containment.resolve_in_bounds("..\\..\\Windows").is_err());
    assert!(containment.resolve_in_bounds("src/../../../root").is_err());
}

#[test]
fn test_path_containment_handles_new_file_creation_within_bounds() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    let containment = PathContainment::new(root);
    let res = containment.resolve_in_bounds("docs/new_doc.md");
    assert!(res.is_ok());
    assert!(res.unwrap().starts_with(root.canonicalize().unwrap_or_else(|_| PathBuf::from(root))));
}
