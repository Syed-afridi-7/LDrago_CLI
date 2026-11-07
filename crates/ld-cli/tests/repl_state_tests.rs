use ld_cli::repl::commands::{parse_repl_command, ReplCommand};
use ld_cli::repl::ReplSession;
use ld_schemas::BrainType;

#[test]
fn test_repl_model_cycling() {
    let mut session = ReplSession::new();
    assert_eq!(session.active_brain, BrainType::Coding);
    assert_eq!(session.prompt_prefix(), "ldrago (coding) > ");

    assert_eq!(session.cycle_model(), BrainType::Design);
    assert_eq!(session.active_brain, BrainType::Design);

    assert_eq!(session.cycle_model(), BrainType::General);
    assert_eq!(session.active_brain, BrainType::General);

    assert_eq!(session.cycle_model(), BrainType::Coding);
    assert_eq!(session.active_brain, BrainType::Coding);
}

#[test]
fn test_parse_slash_commands() {
    assert_eq!(parse_repl_command("/model"), ReplCommand::Model(None));
    assert_eq!(
        parse_repl_command("/model design"),
        ReplCommand::Model(Some("design".to_string()))
    );
    assert_eq!(
        parse_repl_command("  /model   coding  "),
        ReplCommand::Model(Some("coding".to_string()))
    );
    assert_eq!(parse_repl_command("/status"), ReplCommand::Status);
    assert_eq!(parse_repl_command("/clear"), ReplCommand::Clear);
    assert_eq!(parse_repl_command("/help"), ReplCommand::Help);
    assert_eq!(parse_repl_command("/exit"), ReplCommand::Exit);
    assert_eq!(parse_repl_command("exit"), ReplCommand::Exit);
    assert_eq!(parse_repl_command("quit"), ReplCommand::Exit);
    assert_eq!(parse_repl_command("/quit"), ReplCommand::Exit);
    assert_eq!(
        parse_repl_command("write a test"),
        ReplCommand::Prompt("write a test".to_string())
    );
    assert_eq!(
        parse_repl_command(""),
        ReplCommand::Prompt("".to_string())
    );
}

#[test]
fn test_repl_prompt_formatting() {
    let mut session = ReplSession::new();
    assert_eq!(session.prompt_prefix(), "ldrago (coding) > ");

    session.active_brain = BrainType::Design;
    assert_eq!(session.prompt_prefix(), "ldrago (design) > ");

    session.active_brain = BrainType::General;
    assert_eq!(session.prompt_prefix(), "ldrago (general) > ");
}

#[test]
fn test_set_model_by_name() {
    let mut session = ReplSession::new();

    // Coding aliases
    assert!(session.set_model("coding"));
    assert_eq!(session.active_brain, BrainType::Coding);
    assert!(session.set_model("CODER"));
    assert_eq!(session.active_brain, BrainType::Coding);
    assert!(session.set_model("1"));
    assert_eq!(session.active_brain, BrainType::Coding);

    // Design aliases
    assert!(session.set_model("design"));
    assert_eq!(session.active_brain, BrainType::Design);
    assert!(session.set_model("Designer"));
    assert_eq!(session.active_brain, BrainType::Design);
    assert!(session.set_model("MAMBA"));
    assert_eq!(session.active_brain, BrainType::Design);
    assert!(session.set_model("2"));
    assert_eq!(session.active_brain, BrainType::Design);

    // General aliases
    assert!(session.set_model("general"));
    assert_eq!(session.active_brain, BrainType::General);
    assert!(session.set_model("Ollama"));
    assert_eq!(session.active_brain, BrainType::General);
    assert!(session.set_model("3"));
    assert_eq!(session.active_brain, BrainType::General);

    // Invalid model names
    assert!(!session.set_model("unknown_brain"));
    assert!(!session.set_model("gpt-4"));
    assert_eq!(session.active_brain, BrainType::General); // unchanged
}
