//! Agent loop engine connecting model completions, tool execution, and telemetry.

use anyhow::Result;
use ld_models::ModelRouter;
use ld_schemas::{BrainType, CompletionRequest};
use std::time::Instant;

use crate::agent::dispatcher::ToolDispatcher;
use crate::agent::telemetry::record_inference_telemetry;
use crate::agent::tool_parser::parse_tool_invocations;

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
    ///
    /// Evaluates the prompt via the routed model, records inference telemetry in the background,
    /// parses any emitted tool invocations, executes them safely via ToolDispatcher,
    /// and appends the tool execution outputs.
    pub async fn run_turn(&self, brain: BrainType, prompt: &str) -> Result<String> {
        let req = CompletionRequest {
            prompt: prompt.to_string(),
            brain,
            max_tokens: Some(512),
            temperature: Some(0.7),
            stream: false,
        };

        let start = Instant::now();
        let adapter = self.router.route(brain).await?;
        let completion = adapter.complete(&req).await?;
        let elapsed = start.elapsed();

        let tokens = completion.tokens_generated.unwrap_or(32);
        record_inference_telemetry(tokens, elapsed);

        // Check if the brain emitted any tool calls
        let tools = parse_tool_invocations(&completion.text);
        if tools.is_empty() {
            return Ok(completion.text);
        }

        let mut output = completion.text;
        for tool in tools {
            println!("\n  [Executing tool: {:?}]", tool);
            let result = match self.dispatcher.execute(&tool).await {
                Ok(res) => res,
                Err(err) => format!("[ERROR] {err}"),
            };
            output.push_str(&format!("\n[Tool Output]:\n{result}"));
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
