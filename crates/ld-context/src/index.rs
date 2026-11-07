use crate::scanner::ScannedFile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub relative_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub context_excerpt: String,
}

pub struct KeywordIndex<'a> {
    files: &'a [ScannedFile],
}

impl<'a> KeywordIndex<'a> {
    pub fn build(files: &'a [ScannedFile]) -> Self {
        Self { files }
    }

    pub fn search(&self, term: &str, max_results: usize) -> Vec<SearchMatch> {
        let term_lower = term.to_lowercase();
        let mut matches = Vec::new();

        for file in self.files {
            if let Some(content) = &file.content {
                let lines: Vec<&str> = content.lines().collect();
                for (idx, line) in lines.iter().enumerate() {
                    if line.to_lowercase().contains(&term_lower) {
                        let line_number = idx + 1;
                        let line_content = line.to_string();

                        // Build 3-line excerpt
                        let start = idx.saturating_sub(1);
                        let end = (idx + 2).min(lines.len());
                        let context_excerpt = lines[start..end].join("\n");

                        matches.push(SearchMatch {
                            file_path: file.path.clone(),
                            relative_path: file.relative_path.clone(),
                            line_number,
                            line_content,
                            context_excerpt,
                        });

                        if matches.len() >= max_results {
                            return matches;
                        }
                    }
                }
            }
        }

        matches
    }
}
