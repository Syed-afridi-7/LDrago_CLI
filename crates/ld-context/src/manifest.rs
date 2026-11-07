use crate::scanner::ScannedFile;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub total_files: usize,
    pub total_bytes: u64,
    pub languages: BTreeMap<String, usize>,
    pub ascii_tree: String,
}

impl ProjectManifest {
    pub fn build<P: AsRef<Path>>(_root: P, files: &[ScannedFile]) -> Self {
        let total_files = files.len();
        let total_bytes = files.iter().map(|f| f.size_bytes).sum();

        let mut languages_count: HashMap<String, usize> = HashMap::new();
        for file in files {
            let ext = file
                .relative_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let lang = match ext {
                "rs" => "Rust",
                "py" => "Python",
                "json" => "JSON",
                "toml" => "TOML",
                "md" => "Markdown",
                "yaml" | "yml" => "YAML",
                "sh" | "bash" => "Shell",
                "ps1" => "PowerShell",
                "js" => "JavaScript",
                "ts" => "TypeScript",
                "css" => "CSS",
                "html" => "HTML",
                _ => "Other",
            };
            *languages_count.entry(lang.to_string()).or_insert(0) += 1;
        }

        let mut sorted_languages = BTreeMap::new();
        for (lang, count) in languages_count {
            sorted_languages.insert(lang, count);
        }

        let ascii_tree = generate_ascii_tree(files);

        Self {
            total_files,
            total_bytes,
            languages: sorted_languages,
            ascii_tree,
        }
    }
}

fn generate_ascii_tree(files: &[ScannedFile]) -> String {
    // Generate hierarchical representation of relative paths
    let mut tree_lines = Vec::new();
    tree_lines.push(".".to_string());

    let mut seen_dirs: Vec<String> = Vec::new();

    for file in files {
        let rel = file.relative_path.to_string_lossy().replace('\\', "/");
        let parts: Vec<&str> = rel.split('/').collect();

        // Print folder headers if new
        let mut curr_dir = String::new();
        for i in 0..(parts.len().saturating_sub(1)) {
            if !curr_dir.is_empty() {
                curr_dir.push('/');
            }
            curr_dir.push_str(parts[i]);

            if !seen_dirs.contains(&curr_dir) {
                let indent = "  ".repeat(i + 1);
                tree_lines.push(format!("{}├── {}/", indent, parts[i]));
                seen_dirs.push(curr_dir.clone());
            }
        }

        let depth = parts.len();
        let indent = "  ".repeat(depth);
        let file_name = parts.last().unwrap_or(&"");
        tree_lines.push(format!("{}└── {}", indent, file_name));
    }

    tree_lines.join("\n")
}
