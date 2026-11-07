use serde::{Deserialize, Serialize};

/// Structured representation of tool calls extracted from model completions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInvocation {
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    PatchFile {
        path: String,
        target: String,
        replacement: String,
    },
    RunCommand {
        command: String,
    },
    ListFiles {
        path: Option<String>,
    },
}

/// Strip enclosing quotes or backticks from path-like arguments.
fn clean_arg(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('`') && s.ends_with('`') && s.len() >= 2)
        || (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Clean command arguments, stripping optional outer backticks.
fn clean_command(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('`') && s.ends_with('`') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Checks if a line matches `TOOL_END` (optionally wrapped in backticks or markdown).
fn is_tool_end(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('`').trim();
    trimmed == "TOOL_END"
}

/// Strip leading markdown decoration (backticks, bullets, blockquotes).
fn strip_markdown_prefix(s: &str) -> &str {
    s.trim().trim_start_matches(|c| c == '`' || c == '*' || c == '-' || c == '>' || c == ' ')
}

/// Parse deterministic tool protocol invocations from completion text.
///
/// Supported markers:
/// - `TOOL: read_file <path>`
/// - `TOOL: run_command <command>`
/// - `TOOL: list_files <optional_path>`
/// - `TOOL: write_file <path>` (followed by content lines until `TOOL_END` or next `TOOL: `)
/// - `TOOL: patch_file <path>` (followed by `<<<TARGET\n...\n===REPLACE\n...\n>>>` or `TOOL_END`)
pub fn parse_tool_invocations(text: &str) -> Vec<ToolInvocation> {
    let mut tools = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        let stripped = strip_markdown_prefix(line);

        if let Some(rest) = stripped.strip_prefix("TOOL: ") {
            let rest = rest.trim_end_matches('`').trim();
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            let action = parts[0];
            let arg = if parts.len() > 1 { parts[1].trim() } else { "" };

            match action {
                "read_file" => {
                    let cleaned = clean_arg(arg);
                    if !cleaned.is_empty() {
                        tools.push(ToolInvocation::ReadFile {
                            path: cleaned.to_string(),
                        });
                    }
                }
                "run_command" => {
                    let cleaned = clean_command(arg);
                    if !cleaned.is_empty() {
                        tools.push(ToolInvocation::RunCommand {
                            command: cleaned.to_string(),
                        });
                    }
                }
                "list_files" => {
                    let cleaned = clean_arg(arg);
                    let p = if cleaned.is_empty() {
                        None
                    } else {
                        Some(cleaned.to_string())
                    };
                    tools.push(ToolInvocation::ListFiles { path: p });
                }
                "write_file" => {
                    let path = clean_arg(arg).to_string();
                    let mut content_lines = Vec::new();
                    i += 1;
                    while i < lines.len() {
                        let cur_line = lines[i].trim();
                        let cur_stripped = strip_markdown_prefix(cur_line);
                        if is_tool_end(cur_line) || cur_stripped.starts_with("TOOL: ") {
                            if cur_stripped.starts_with("TOOL: ") {
                                i -= 1;
                            }
                            break;
                        }
                        content_lines.push(lines[i]);
                        i += 1;
                    }
                    tools.push(ToolInvocation::WriteFile {
                        path,
                        content: content_lines.join("\n"),
                    });
                }
                "patch_file" => {
                    let path = clean_arg(arg).to_string();
                    let mut target_lines = Vec::new();
                    let mut replace_lines = Vec::new();
                    let mut state = 0; // 0: before target, 1: in target, 2: in replace
                    i += 1;
                    while i < lines.len() {
                        let cur_line = lines[i].trim();
                        let cur_stripped = strip_markdown_prefix(cur_line);
                        if cur_stripped.starts_with("TOOL: ") {
                            i -= 1;
                            break;
                        }
                        if is_tool_end(cur_line) || cur_line == ">>>" || cur_line.starts_with(">>>") {
                            break;
                        }
                        if cur_line.starts_with("<<<TARGET") {
                            state = 1;
                            i += 1;
                            continue;
                        }
                        if cur_line.starts_with("===REPLACE") {
                            state = 2;
                            i += 1;
                            continue;
                        }
                        match state {
                            1 => target_lines.push(lines[i]),
                            2 => replace_lines.push(lines[i]),
                            _ => {}
                        }
                        i += 1;
                    }
                    tools.push(ToolInvocation::PatchFile {
                        path,
                        target: target_lines.join("\n"),
                        replacement: replace_lines.join("\n"),
                    });
                }
                _ => {}
            }
        }
        i += 1;
    }

    tools
}
