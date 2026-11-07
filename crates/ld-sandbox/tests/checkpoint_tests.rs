use ld_sandbox::checkpoint::CheckpointManager;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_checkpoint_lifecycle_create_list_restore() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // 1. Setup workspace with initial files
    let file1 = root.join("src/lib.rs");
    let file2 = root.join("README.md");
    fs::create_dir_all(root.join("src")).expect("mkdir src");
    fs::write(&file1, "// initial lib").expect("write lib");
    fs::write(&file2, "# Initial Readme").expect("write readme");

    let manager = CheckpointManager::new(root);

    // 2. Create checkpoint
    let meta = manager
        .create_checkpoint("initial commit snapshot", &[file1.clone(), file2.clone()])
        .expect("create checkpoint");
    assert_eq!(meta.file_count, 2);
    assert_eq!(meta.description, "initial commit snapshot");

    // 3. List checkpoints
    let list = manager.list_checkpoints().expect("list checkpoints");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, meta.id);

    // 4. Mutate files
    fs::write(&file1, "// mutated corrupted lib").expect("write corrupted");
    fs::write(&file2, "# Altered Readme").expect("write altered");
    assert_eq!(fs::read_to_string(&file1).expect("read"), "// mutated corrupted lib");

    // 5. Restore checkpoint
    let restored = manager.restore_checkpoint(&meta.id).expect("restore checkpoint");
    assert_eq!(restored, 2);

    // 6. Verify restored contents
    assert_eq!(fs::read_to_string(&file1).expect("read"), "// initial lib");
    assert_eq!(fs::read_to_string(&file2).expect("read"), "# Initial Readme");
}
