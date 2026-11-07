use ld_cli::tui::{ActivePane, AppState};
use ld_schemas::BrainType;

#[test]
fn test_default_state_initialization() {
    let app = AppState::new();

    assert_eq!(app.active_pane, ActivePane::Engines);
    assert_eq!(app.selected_brain, BrainType::Coding);
    assert!(app.input.is_empty());
    assert!(app.messages.is_empty());
    assert!(!app.coder_healthy);
    assert!(!app.designer_healthy);
    assert!(!app.ollama_healthy);
    assert_eq!(app.tokens_per_sec, 0.0);
    assert_eq!(app.active_checkpoints, 0);
    assert!(app.running);
}

#[test]
fn test_cycle_focus() {
    let mut app = AppState::new();
    assert_eq!(app.active_pane, ActivePane::Engines);

    // Engines -> Chat
    app.cycle_focus();
    assert_eq!(app.active_pane, ActivePane::Chat);

    // Chat -> Telemetry
    app.cycle_focus();
    assert_eq!(app.active_pane, ActivePane::Telemetry);

    // Telemetry -> Engines
    app.cycle_focus();
    assert_eq!(app.active_pane, ActivePane::Engines);
}

#[test]
fn test_handle_char_delete_char_and_submit_input() {
    let mut app = AppState::new();

    // Input buffer manipulation
    app.handle_char('H');
    app.handle_char('e');
    app.handle_char('y');
    assert_eq!(app.input, "Hey");

    // Deletion
    app.delete_char();
    assert_eq!(app.input, "He");

    app.handle_char('l');
    app.handle_char('l');
    app.handle_char('o');
    assert_eq!(app.input, "Hello");

    // Submit non-empty input
    let submitted = app.submit_input();
    assert_eq!(submitted, Some("Hello".to_string()));
    assert!(app.input.is_empty());

    // Submit empty input returns None
    let empty_submitted = app.submit_input();
    assert_eq!(empty_submitted, None);

    // Submit whitespace-only input returns None and clears buffer
    app.handle_char(' ');
    app.handle_char(' ');
    assert_eq!(app.submit_input(), None);
    assert!(app.input.is_empty());

    // delete_char on empty buffer does not panic
    app.delete_char();
    assert!(app.input.is_empty());
}

#[test]
fn test_add_message_and_append_to_last_message() {
    let mut app = AppState::new();

    // Add first message
    app.add_message("User", "What is L-Drago?");
    assert_eq!(app.messages.len(), 1);
    assert_eq!(app.messages[0].0, "User");
    assert_eq!(app.messages[0].1, "What is L-Drago?");

    // Add assistant turn for streaming
    app.add_message("Assistant", "");
    assert_eq!(app.messages.len(), 2);
    assert_eq!(app.messages[1].0, "Assistant");
    assert_eq!(app.messages[1].1, "");

    // Streaming token chunks
    app.append_to_last_message("L-Drago is ");
    app.append_to_last_message("a multi-engine AI orchestrator.");
    assert_eq!(app.messages[1].1, "L-Drago is a multi-engine AI orchestrator.");

    // Calling append on empty messages does not panic
    let mut empty_app = AppState::new();
    empty_app.append_to_last_message("No crash");
    assert!(empty_app.messages.is_empty());
}

#[test]
fn test_set_health_and_quit() {
    let mut app = AppState::new();

    assert!(!app.coder_healthy);
    assert!(!app.designer_healthy);
    assert!(!app.ollama_healthy);
    assert!(app.running);

    app.set_health(true, false, true);
    assert!(app.coder_healthy);
    assert!(!app.designer_healthy);
    assert!(app.ollama_healthy);

    app.quit();
    assert!(!app.running);
}
