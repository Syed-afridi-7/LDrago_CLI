use ld_cli::agent::tool_parser::{parse_tool_invocations, ToolInvocation};

#[test]
fn test_parse_read_file() {
    let input = "Let me read the file:\nTOOL: read_file src/main.rs\nNow I will check it.";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::ReadFile {
            path: "src/main.rs".to_string()
        }
    );
}

#[test]
fn test_parse_run_command() {
    let input = "I will check the status:\nTOOL: run_command cargo test --workspace";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::RunCommand {
            command: "cargo test --workspace".to_string()
        }
    );
}

#[test]
fn test_parse_write_file() {
    let input = "TOOL: write_file test.txt\nHello world!\nMultiple lines\nTOOL_END";
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    match &tools[0] {
        ToolInvocation::WriteFile { path, content } => {
            assert_eq!(path, "test.txt");
            assert_eq!(content, "Hello world!\nMultiple lines");
        }
        _ => panic!("Expected WriteFile"),
    }
}

#[test]
fn test_parse_list_files() {
    let input_no_path = "TOOL: list_files";
    let tools = parse_tool_invocations(input_no_path);
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0], ToolInvocation::ListFiles { path: None });

    let input_with_path = "TOOL: list_files crates/ld-cli";
    let tools = parse_tool_invocations(input_with_path);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::ListFiles {
            path: Some("crates/ld-cli".to_string())
        }
    );
}

#[test]
fn test_parse_multiple_tools_interleaved_with_prose() {
    let input = r#"
I will start by listing the project files to see the directory layout.
TOOL: list_files

Now that I know what files exist, let me inspect Cargo.toml:
TOOL: read_file Cargo.toml

Let's create a new helper file:
TOOL: write_file src/helper.rs
pub fn helper() -> bool {
    true
}
TOOL_END

Now let's run the tests to verify:
TOOL: run_command cargo test --workspace

Done! All steps planned.
"#;
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 4);
    assert_eq!(tools[0], ToolInvocation::ListFiles { path: None });
    assert_eq!(
        tools[1],
        ToolInvocation::ReadFile {
            path: "Cargo.toml".to_string()
        }
    );
    assert_eq!(
        tools[2],
        ToolInvocation::WriteFile {
            path: "src/helper.rs".to_string(),
            content: "pub fn helper() -> bool {\n    true\n}".to_string(),
        }
    );
    assert_eq!(
        tools[3],
        ToolInvocation::RunCommand {
            command: "cargo test --workspace".to_string(),
        }
    );
}

#[test]
fn test_parse_patch_file() {
    let input = r#"
Let me patch the configuration:
TOOL: patch_file src/config.rs
<<<TARGET
pub fn is_enabled() -> bool {
    false
}
===REPLACE
pub fn is_enabled() -> bool {
    true
}
>>>
TOOL_END
"#;
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 1);
    assert_eq!(
        tools[0],
        ToolInvocation::PatchFile {
            path: "src/config.rs".to_string(),
            target: "pub fn is_enabled() -> bool {\n    false\n}".to_string(),
            replacement: "pub fn is_enabled() -> bool {\n    true\n}".to_string(),
        }
    );
}

#[test]
fn test_parse_write_file_terminated_by_next_tool() {
    let input = r#"
TOOL: write_file src/temp.txt
line 1
line 2
TOOL: read_file src/temp.txt
"#;
    let tools = parse_tool_invocations(input);
    assert_eq!(tools.len(), 2);
    assert_eq!(
        tools[0],
        ToolInvocation::WriteFile {
            path: "src/temp.txt".to_string(),
            content: "line 1\nline 2".to_string(),
        }
    );
    assert_eq!(
        tools[1],
        ToolInvocation::ReadFile {
            path: "src/temp.txt".to_string(),
        }
    );
}
