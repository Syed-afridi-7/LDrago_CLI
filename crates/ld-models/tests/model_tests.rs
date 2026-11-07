use ld_models::*;
use ld_schemas::{BrainType, Role, Turn};
use std::sync::Arc;

#[test]
fn test_adapter_capabilities() {
    let coder = CustomBrainAdapter::new("http://127.0.0.1:8741", BrainType::Coding);
    let caps = coder.capabilities();
    assert_eq!(caps.alias, "coder-local");
    assert_eq!(caps.brain, Some(BrainType::Coding));
    assert!(caps.capabilities.contains(&"coding".to_string()));

    let designer = CustomBrainAdapter::new("http://127.0.0.1:8741", BrainType::Design);
    let caps = designer.capabilities();
    assert_eq!(caps.alias, "designer-local");
    assert_eq!(caps.brain, Some(BrainType::Design));

    let ollama = OllamaAdapter::new("http://127.0.0.1:11434", "llama3");
    let caps = ollama.capabilities();
    assert_eq!(caps.alias, "ollama-general");
    assert_eq!(caps.brain, Some(BrainType::General));
}

#[tokio::test]
async fn test_model_router_registration_and_listing() {
    let mut router = ModelRouter::new();
    let coder = Arc::new(CustomBrainAdapter::new("http://127.0.0.1:8741", BrainType::Coding));
    let designer = Arc::new(CustomBrainAdapter::new("http://127.0.0.1:8741", BrainType::Design));
    let ollama = Arc::new(OllamaAdapter::new("http://127.0.0.1:11434", "llama3"));

    router.register(coder);
    router.register(designer);
    router.register(ollama);

    let caps = router.list_capabilities();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps[0].alias, "coder-local");
    assert_eq!(caps[1].alias, "designer-local");
    assert_eq!(caps[2].alias, "ollama-general");
}

#[test]
fn test_model_switch_context_and_provenance() {
    let mut ctx = ModelSwitchContext::new();

    ctx.add_turn(Turn::new(Role::User, "Design the navbar"));
    ctx.add_turn(Turn::new(Role::Assistant, "Here is the navbar component"));

    ctx.record_switch(
        BrainType::Design,
        BrainType::Coding,
        "ldrago-designer-ssm",
        "ldrago-coder-transformer",
        "User requested code implementation",
        true,
    );

    assert_eq!(ctx.turns.len(), 2);
    assert_eq!(ctx.switch_history.len(), 1);
    assert_eq!(ctx.switch_history[0].from_brain, BrainType::Design);
    assert_eq!(ctx.switch_history[0].to_brain, BrainType::Coding);
    assert!(ctx.switch_history[0].user_confirmed);

    let formatted = ctx.format_prompt_for_target(Some("You are an expert coder."), "Implement tests");
    assert!(formatted.contains("System: You are an expert coder."));
    assert!(formatted.contains("User: Design the navbar"));
    assert!(formatted.contains("Assistant: Here is the navbar component"));
    assert!(formatted.contains("User: Implement tests"));
}
