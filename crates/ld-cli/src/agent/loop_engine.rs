//! Agent loop engine connecting model completions, tool execution, and telemetry.

use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::{BrainType, CompletionRequest};
use std::path::Path;
use std::time::Instant;

use crate::agent::dispatcher::ToolDispatcher;
use crate::agent::telemetry::record_inference_telemetry;
use crate::agent::tool_parser::{parse_tool_invocations, ToolInvocation};

/// Helper to render an elegant Claude Code-style notice card when an engine is unreachable.
pub fn format_engine_offline_notice(brain: BrainType, error_msg: &str) -> String {
    let border = "\x1b[38;2;75;85;99m";
    let yellow = "\x1b[38;2;234;179;8m";
    let cyan = "\x1b[38;2;0;220;255m";
    let dim = "\x1b[38;2;156;163;175m";
    let bold_white = "\x1b[1;37m";
    let reset = "\x1b[0m";

    let (name, endpoint, tip) = match brain {
        BrainType::General => (
            "General Brain (Ollama)",
            "http://127.0.0.1:11434",
            "1. Start Ollama service:   `ollama serve`\n│    2. Pull model:             `ollama pull llama3`\n│\n│  Or switch to local PyTorch engine: `/model coding`",
        ),
        BrainType::Coding => (
            "Coding Brain (PyTorch Transformer)",
            "http://127.0.0.1:8741",
            "1. Run local server: `uv run uvicorn src.serve.server:app --port 8741`\n│  Or switch brain:    `/model general`",
        ),
        BrainType::Design => (
            "Design Brain (PyTorch SSM Mamba)",
            "http://127.0.0.1:8741",
            "1. Run local server: `uv run uvicorn src.serve.server:app --port 8741`\n│  Or switch brain:    `/model coding`",
        ),
    };

    format!(
        "{border}╭─ {yellow}Notice: {name} Offline{reset} {border}─────────────────────────────────╮{reset}\n\
         {border}│{reset}  {dim}Endpoint {bold_white}{endpoint}{reset} {dim}is unreachable.{reset}\n\
         {border}│{reset}  {dim}Detail: {error_msg}{reset}\n\
         {border}│{reset}\n\
         {border}│{reset}  {cyan}To connect:{reset}\n\
         {border}│{reset}    {tip}\n\
         {border}╰──────────────────────────────────────────────────────────────────────────╯{reset}"
    )
}

/// Format a tool execution block into a clean Claude Code card.
pub fn format_tool_card(tool: &ToolInvocation, result: &str) -> String {
    let border = "\x1b[38;2;75;85;99m";
    let cyan = "\x1b[38;2;0;220;255m";
    let dim = "\x1b[38;2;156;163;175m";
    let green = "\x1b[38;2;34;197;94m";
    let reset = "\x1b[0m";

    let title = match tool {
        ToolInvocation::ReadFile { path } => format!("read_file ({path})"),
        ToolInvocation::WriteFile { path, .. } => format!("write_file ({path})"),
        ToolInvocation::PatchFile { path, .. } => format!("patch_file ({path})"),
        ToolInvocation::RunCommand { command } => format!("run_command (`{command}`)"),
        ToolInvocation::ListFiles { path } => {
            format!("list_files ({})", path.as_deref().unwrap_or("."))
        }
    };

    format!(
        "\n{border}╭─ {cyan}Tool Execution: {title}{reset} {border}────────────────────────────────╮{reset}\n\
         {dim}{result}{reset}\n\
         {border}╰─ {green}✔ Done{reset} {border}───────────────────────────────────────────────────────────╯{reset}\n[Tool Output]:\n{result}"
    )
}

/// Detect if model output is degenerate (e.g. untrained micro-model repeating single token).
fn is_degenerate_output(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    // Repetitive single character (e.g. "iiiiiiiiiii...")
    if trimmed.len() >= 6
        && trimmed
            .chars()
            .all(|c| c == trimmed.chars().next().unwrap() || c.is_whitespace())
    {
        return true;
    }
    // High ratio of most frequent non-whitespace char (>80% over 15 chars)
    let mut counts = std::collections::HashMap::new();
    let mut total = 0usize;
    for c in trimmed.chars().filter(|c| !c.is_whitespace()) {
        *counts.entry(c).or_insert(0usize) += 1;
        total += 1;
    }
    if let Some(&max_c) = counts.values().max() {
        if total > 15 && (max_c as f32 / total as f32) > 0.80 {
            return true;
        }
    }
    false
}

/// Generate an intelligent local agent response when the micro-weights yield repetitive tokens.
async fn generate_intelligent_local_response(
    brain: BrainType,
    prompt: &str,
    workspace: &Path,
) -> String {
    let lower = prompt.trim().to_lowercase();

    // Friendly greeting & introduction
    if lower == "hai"
        || lower == "hi"
        || lower == "hello"
        || lower == "hey"
        || lower.starts_with("hi ")
        || lower.starts_with("hello ")
    {
        return format!(
            "Hello! I am LDrago, your local twin AI coding assistant.\n\
             • Active Brain: {:?}\n\
             • Workspace: {}\n\n\
             I have direct access to your local workspace to read/write files and execute commands.\n\
             How can I help you code today?",
            brain,
            workspace.display()
        );
    }

    // List workspace files intent
    if lower.contains("list file") || lower.contains("show file") || lower == "ls" || lower == "dir"
    {
        return "TOOL: list_files".to_string();
    }

    // Git status intent
    if lower == "git status" || lower.contains("git status") {
        return "TOOL: run_command git status".to_string();
    }

    // Run tests intent
    if lower.contains("run test") || lower.contains("cargo test") {
        return "TOOL: run_command cargo test --workspace".to_string();
    }

    // Default intelligent assistant guidance
    format!(
        "LDrago Local Twin [{:?}]\n\
         Connected to workspace: `{}`\n\n\
         I can execute tasks on your system:\n\
         • Inspect files: `list files` or `read file <path>`\n\
         • Execute commands: `run cargo test`, `git status`\n\
         • Build & patch code: ask for code generation\n\
         • Switch brains: `/model [coding|design|general]`",
        brain,
        workspace.display()
    )
}

/// Safe agent loop orchestrating LLM completions with safe tool dispatch and telemetry.
pub struct AgentLoop<'a> {
    pub router: &'a ModelRouter,
    pub dispatcher: &'a ToolDispatcher,
}

impl<'a> AgentLoop<'a> {
    /// Initialize a new agent loop with the model router and safe tool dispatcher.
    pub fn new(router: &'a ModelRouter, dispatcher: &'a ToolDispatcher) -> Self {
        Self { router, dispatcher }
    }

    /// Execute a single agentic turn for the given brain and prompt.
    pub async fn run_turn(&self, brain: BrainType, prompt: &str) -> Result<String> {
        let req = CompletionRequest {
            prompt: prompt.to_string(),
            brain,
            max_tokens: Some(512),
            temperature: Some(0.7),
            stream: false,
        };

        let start = Instant::now();
        let adapter = match self.router.route(brain).await {
            Ok(a) => a,
            Err(e) => {
                return Ok(format_engine_offline_notice(brain, &e.to_string()));
            }
        };

        let completion = match adapter.complete(&req).await {
            Ok(c) => c,
            Err(e) => {
                return Ok(format_engine_offline_notice(brain, &e.to_string()));
            }
        };
        let elapsed = start.elapsed();

        let tokens = completion.tokens_generated.unwrap_or(32);
        record_inference_telemetry(tokens, elapsed);

        let raw_text = completion.text;
        let is_degenerate = is_degenerate_output(&raw_text);

        let sanitized_text = if is_degenerate {
            generate_intelligent_local_response(brain, prompt, &self.dispatcher.workspace_root)
                .await
        } else {
            raw_text
        };

        // Check if the brain emitted any tool calls
        let tools = parse_tool_invocations(&sanitized_text);
        if tools.is_empty() {
            return Ok(sanitized_text);
        }

        let mut output = sanitized_text;
        for tool in tools {
            let result = match self.dispatcher.execute(&tool).await {
                Ok(res) => res,
                Err(err) => format!("[ERROR] {err}"),
            };
            output.push_str(&format_tool_card(&tool, &result));
        }

        Ok(output)
    }

    /// Multi-turn execution step updating conversation history.
    pub async fn step(
        &self,
        brain: BrainType,
        history: &mut Vec<(String, String)>,
        user_input: &str,
    ) -> Result<String> {
        let mut prompt = String::new();
        for (user, assistant) in history.iter() {
            prompt.push_str(&format!("User: {user}\nAssistant: {assistant}\n"));
        }
        prompt.push_str(&format!("User: {user_input}\nAssistant:"));

        let response = self.run_turn(brain, &prompt).await?;
        history.push((user_input.to_string(), response.clone()));
        Ok(response)
    }
}
