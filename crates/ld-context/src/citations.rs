use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub relative_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub verified: bool,
    pub snippet: String,
    pub file_url: String,
}

pub struct CitationExtractor {
    root: PathBuf,
    regex: Regex,
}

impl CitationExtractor {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        // Matches [path/to/file#L1-L10] or [path/to/file#L5]
        let regex = Regex::new(r"\[([a-zA-Z0-9_\-./\\]+)#L(\d+)(?:-L(\d+))?\]").unwrap();
        Self {
            root: root.as_ref().to_path_buf(),
            regex,
        }
    }

    pub fn extract_and_verify(&self, text: &str) -> Vec<Citation> {
        let mut citations = Vec::new();

        for cap in self.regex.captures_iter(text) {
            let rel_path_raw = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let rel_path = rel_path_raw.replace('\\', "/");
            let start_line: usize = cap
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(1);
            let end_line: usize = cap
                .get(3)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(start_line);

            let abs_path = self.root.join(&rel_path);
            let mut verified = false;
            let mut snippet = String::new();

            if abs_path.is_file() {
                if let Ok(content) = fs::read_to_string(&abs_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    if start_line > 0 && start_line <= lines.len() {
                        let actual_end = end_line.min(lines.len());
                        if start_line <= actual_end {
                            verified = true;
                            snippet = lines[(start_line - 1)..actual_end].join("\n");
                        }
                    }
                }
            }

            let file_url = format!(
                "file:///{}/{}#L{}-L{}",
                self.root.to_string_lossy().replace('\\', "/").trim_end_matches('/'),
                rel_path.trim_start_matches('/'),
                start_line,
                end_line
            );

            citations.push(Citation {
                relative_path: rel_path,
                start_line,
                end_line,
                verified,
                snippet,
                file_url,
            });
        }

        citations
    }
}
