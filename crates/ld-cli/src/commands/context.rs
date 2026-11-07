use anyhow::{Context, Result};
use clap::Subcommand;
use ld_context::{KeywordIndex, ProjectManifest, ScanOptions, WorkspaceScanner};
use std::env;

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum ContextCommands {
    /// Print an ASCII tree of scanned workspace files
    Tree,
    /// Search codebase for symbols or keywords
    Search {
        /// Term to search for
        term: String,
        /// Maximum number of search results
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// Print project manifest with language stats and file metrics
    Manifest,
}

pub fn execute_context(action: ContextCommands) -> Result<()> {
    let current_dir = env::current_dir().context("get current working directory")?;
    let scanner = WorkspaceScanner::new(ScanOptions::default());
    let files = scanner.scan(&current_dir).context("scan workspace")?;

    match action {
        ContextCommands::Tree => {
            let manifest = ProjectManifest::build(&current_dir, &files);
            println!("LDrago Workspace Tree ({})", current_dir.display());
            println!("--------------------------------------------------------");
            println!("{}", manifest.ascii_tree);
            println!("--------------------------------------------------------");
            println!("Total scanned files: {}", manifest.total_files);
        }
        ContextCommands::Search { term, limit } => {
            let index = KeywordIndex::build(&files);
            let matches = index.search(&term, limit);

            println!("Search Results for \"{}\" ({} matches):", term, matches.len());
            println!("--------------------------------------------------------");
            if matches.is_empty() {
                println!("No matches found in scanned text files.");
            } else {
                for m in matches {
                    println!(
                        "[{}:{}] {}",
                        m.relative_path.display(),
                        m.line_number,
                        m.line_content.trim()
                    );
                    println!("  Context:\n{}", indent_excerpt(&m.context_excerpt));
                    println!();
                }
            }
        }
        ContextCommands::Manifest => {
            let manifest = ProjectManifest::build(&current_dir, &files);
            println!("LDrago Project Manifest ({})", current_dir.display());
            println!("--------------------------------------------------------");
            println!("Total Files : {}", manifest.total_files);
            println!("Total Size  : {} KB", manifest.total_bytes / 1024);
            println!("Languages   :");
            for (lang, count) in &manifest.languages {
                println!("  • {:<12}: {} files", lang, count);
            }
            println!("--------------------------------------------------------");
        }
    }

    Ok(())
}

fn indent_excerpt(excerpt: &str) -> String {
    excerpt
        .lines()
        .map(|l| format!("    | {}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
