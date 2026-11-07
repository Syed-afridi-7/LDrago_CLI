use ld_context::citations::CitationExtractor;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_citation_extraction_and_verification() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/model.rs"),
        "line 1\nline 2\nline 3\nline 4\nline 5\n",
    )
    .unwrap();

    let text = "As shown in [src/model.rs#L2-L4], the model definition handles RoPE embeddings.";
    let extractor = CitationExtractor::new(root);
    let citations = extractor.extract_and_verify(text);

    assert_eq!(citations.len(), 1);
    let c = &citations[0];
    assert_eq!(c.relative_path, "src/model.rs");
    assert_eq!(c.start_line, 2);
    assert_eq!(c.end_line, 4);
    assert!(c.verified);
    assert_eq!(c.snippet, "line 2\nline 3\nline 4");
    assert!(c.file_url.contains("src/model.rs#L2-L4"));
}

#[test]
fn test_unverified_citation_when_file_missing_or_out_of_bounds() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    let text = "See non-existent [src/missing.rs#L10-L20] here.";
    let extractor = CitationExtractor::new(root);
    let citations = extractor.extract_and_verify(text);

    assert_eq!(citations.len(), 1);
    assert!(!citations[0].verified);
    assert!(citations[0].snippet.is_empty());
}
