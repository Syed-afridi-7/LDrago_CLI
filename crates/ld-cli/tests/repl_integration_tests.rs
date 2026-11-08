use async_trait::async_trait;
use ld_cli::agent::dispatcher::ToolDispatcher;
use ld_cli::agent::loop_engine::AgentLoop;
use ld_cli::agent::telemetry::get_telemetry_stats;
use ld_cli::repl::prompt::run_repl_loop;
use ld_models::{ModelAdapter, ModelError, ModelRouter};
use ld_schemas::{
    BrainType, CompletionRequest, CompletionResponse, HealthStatus, ModelCapabilities, PrivacyClass,
};
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct MockModelAdapter {
    brain: BrainType,
    responses: Vec<String>,
    call_count: AtomicUsize,
}

impl MockModelAdapter {
    fn new(brain: BrainType, responses: Vec<String>) -> Self {
        Self {
            brain,
            responses,
            call_count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl ModelAdapter for MockModelAdapter {
    async fn health_check(&self) -> Result<HealthStatus, ModelError> {
        Ok(HealthStatus::Healthy)
    }

    async fn complete(&self, _req: &CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        let text = if idx < self.responses.len() {
            self.responses[idx].clone()
        } else {
            self.responses.last().cloned().unwrap_or_default()
        };
        Ok(CompletionResponse {
            text,
            brain: self.brain,
            tokens_generated: Some(25),
        })
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            alias: format!("{}-mock", self.brain),
            endpoint: "http://mock".to_string(),
            model_name: "mock-model".to_string(),
            capabilities: vec!["code".to_string()],
            context_window: 2048,
            privacy: PrivacyClass::Local,
            brain: Some(self.brain),
        }
    }
}

#[tokio::test]
async fn test_agent_loop_initialization_and_turn_without_tools() {
    let mock = Arc::new(MockModelAdapter::new(
        BrainType::Coding,
        vec!["I am ready to write code.".to_string()],
    ));

    let mut router = ModelRouter::new();
    router.register(mock);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());
    let agent = AgentLoop::new(&router, &dispatcher);

    let initial_telemetry = get_telemetry_stats();
    let result = agent
        .run_turn(BrainType::Coding, "Hello, can you help me?")
        .await
        .unwrap();

    assert_eq!(result, "I am ready to write code.");
    let updated_telemetry = get_telemetry_stats();
    assert!(updated_telemetry.total_tokens >= initial_telemetry.total_tokens + 25);
}

#[tokio::test]
async fn test_agent_loop_turn_with_single_tool_invocation() {
    let tool_response = r#"I will write the file now.
TOOL: write_file created.txt
Hello world from AgentLoop!
TOOL_END
All done!"#
        .to_string();

    let mock = Arc::new(MockModelAdapter::new(BrainType::Coding, vec![tool_response]));
    let mut router = ModelRouter::new();
    router.register(mock);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());
    let agent = AgentLoop::new(&router, &dispatcher);

    let result = agent
        .run_turn(BrainType::Coding, "Create created.txt")
        .await
        .unwrap();

    // Verify disk side effect
    let created_file = temp.path().join("created.txt");
    assert!(created_file.exists());
    let content = std::fs::read_to_string(&created_file).unwrap();
    assert_eq!(content, "Hello world from AgentLoop!");

    // Verify tool output was appended
    assert!(result.contains("I will write the file now."));
    assert!(result.contains("[Tool Output]:\n[OK] Successfully wrote 27 bytes to 'created.txt'."));
}

#[tokio::test]
async fn test_agent_loop_multi_tool_in_single_turn() {
    let multi_tool_response = r#"Creating file and reading it back.
TOOL: write_file batch.txt
Batch Content 42
TOOL_END
TOOL: read_file batch.txt"#
        .to_string();

    let mock = Arc::new(MockModelAdapter::new(
        BrainType::Coding,
        vec![multi_tool_response],
    ));
    let mut router = ModelRouter::new();
    router.register(mock);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());
    let agent = AgentLoop::new(&router, &dispatcher);

    let result = agent
        .run_turn(BrainType::Coding, "Write and read batch.txt")
        .await
        .unwrap();

    assert!(result.contains("[Tool Output]:\n[OK] Successfully wrote 16 bytes to 'batch.txt'."));
    assert!(result.contains("[Tool Output]:\nBatch Content 42"));
}

#[tokio::test]
async fn test_agent_loop_multi_turn_execution_appending_tool_outputs() {
    let turn1 = "TOOL: write_file step.txt\nStep 1 data\nTOOL_END".to_string();
    let turn2 = "TOOL: read_file step.txt".to_string();
    let turn3 = "Final verification complete.".to_string();

    let mock = Arc::new(MockModelAdapter::new(
        BrainType::Coding,
        vec![turn1, turn2, turn3],
    ));
    let mut router = ModelRouter::new();
    router.register(mock);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());
    let agent = AgentLoop::new(&router, &dispatcher);

    // Turn 1: Write file
    let res1 = agent
        .run_turn(BrainType::Coding, "Create step file")
        .await
        .unwrap();
    assert!(res1.contains("[Tool Output]:\n[OK] Successfully wrote 11 bytes to 'step.txt'."));

    // Turn 2: Read file
    let res2 = agent
        .run_turn(BrainType::Coding, "Read step file back")
        .await
        .unwrap();
    assert!(res2.contains("[Tool Output]:\nStep 1 data"));

    // Turn 3: Pure prose, no tools
    let res3 = agent
        .run_turn(BrainType::Coding, "Confirm completion")
        .await
        .unwrap();
    assert_eq!(res3, "Final verification complete.");
    assert!(!res3.contains("[Tool Output]"));
}

#[tokio::test]
async fn test_agent_loop_step_with_history() {
    let mock = Arc::new(MockModelAdapter::new(
        BrainType::Coding,
        vec!["Step 1 response".to_string(), "Step 2 response".to_string()],
    ));
    let mut router = ModelRouter::new();
    router.register(mock);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());
    let agent = AgentLoop::new(&router, &dispatcher);

    let mut history = Vec::new();
    let r1 = agent
        .step(BrainType::Coding, &mut history, "first prompt")
        .await
        .unwrap();
    assert_eq!(r1, "Step 1 response");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0], ("first prompt".to_string(), "Step 1 response".to_string()));

    let r2 = agent
        .step(BrainType::Coding, &mut history, "second prompt")
        .await
        .unwrap();
    assert_eq!(r2, "Step 2 response");
    assert_eq!(history.len(), 2);
    assert_eq!(history[1], ("second prompt".to_string(), "Step 2 response".to_string()));
}

#[tokio::test]
async fn test_repl_runner_slash_commands_and_prompts() {
    let mock_coder = Arc::new(MockModelAdapter::new(
        BrainType::Coding,
        vec!["I coded it.".to_string()],
    ));
    let mock_designer = Arc::new(MockModelAdapter::new(
        BrainType::Design,
        vec!["I designed it.".to_string()],
    ));

    let mut router = ModelRouter::new();
    router.register(mock_coder);
    router.register(mock_designer);

    let temp = tempdir().unwrap();
    let dispatcher = ToolDispatcher::new(temp.path().to_path_buf());

    let input_script = "/status\n/help\n/model design\ncreate design layout\n/exit\n";
    let reader = Cursor::new(input_script.as_bytes());
    let mut output = Vec::new();

    run_repl_loop(&router, &dispatcher, reader, &mut output)
        .await
        .unwrap();

    let output_str = String::from_utf8(output).unwrap();

    // Check banner & prompt
    assert!(output_str.contains(">_ LDrago"));
    assert!(output_str.contains("ldrago (coding) > "));

    // Check /status response
    assert!(output_str.contains("Active Brain:    Coding (Local Transformer)"));
    assert!(output_str.contains("Contained (Zero-Trust Policy Active)"));

    // Check /help response
    assert!(output_str.contains("/model [coding|design|general]"));
    assert!(output_str.contains("/status"));

    // Check /model switch
    assert!(output_str.contains("Active model switched to Design"));
    assert!(output_str.contains("ldrago (design) > "));

    // Check prompt completion response from Design brain
    assert!(output_str.contains("I designed it."));

    // Check exit
    assert!(output_str.contains("Exiting LDrago."));
}
