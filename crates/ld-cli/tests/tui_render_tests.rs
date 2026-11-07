use ld_cli::tui::{render, render_to_terminal, ActivePane, AppState};
use ld_schemas::BrainType;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// Convert TestBackend buffer into a plain text representation.
fn buffer_to_string(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn test_headless_render_default_state() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to initialize TestBackend terminal");
    let app = AppState::new();

    terminal
        .draw(|f| render(f, &app))
        .expect("Terminal draw failed for default state");

    let output = buffer_to_string(&terminal);

    // Verify key UI elements specified in Task 2
    assert!(
        output.contains("LDRAGO"),
        "Rendered buffer must contain 'LDRAGO', found:\n{output}"
    );
    assert!(
        output.contains("Coding Brain"),
        "Rendered buffer must contain 'Coding Brain', found:\n{output}"
    );
    assert!(
        output.contains("PyTorch SSM"),
        "Rendered buffer must contain 'PyTorch SSM', found:\n{output}"
    );
    assert!(
        output.contains("Telemetry"),
        "Rendered buffer must contain 'Telemetry', found:\n{output}"
    );

    // Verify detailed components
    assert!(output.contains("PyTorch Transformer"));
    assert!(output.contains("Ollama Adapter"));
    assert!(output.contains("Inference Speed:"));
    assert!(output.contains("Active Checkpoints:"));
    assert!(output.contains("Policy Engine:"));
    assert!(output.contains("Sandbox:"));
    assert!(output.contains("[Tab]"));
    assert!(output.contains("Switch Pane"));
    assert!(output.contains("[Esc]"));
    assert!(output.contains("Quit"));
}

#[test]
fn test_headless_render_populated_state() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to initialize TestBackend terminal");
    let mut app = AppState::new();

    // Populate app state
    app.active_pane = ActivePane::Chat;
    app.selected_brain = BrainType::Coding;
    app.set_health(true, true, true);
    app.tokens_per_sec = 64.2;
    app.active_checkpoints = 4;
    app.handle_char('p');
    app.handle_char('l');
    app.handle_char('a');
    app.handle_char('n');
    app.add_message("User", "Generate an architecture design");
    app.add_message("Assistant", "Architecture draft: Modular pipeline");

    terminal
        .draw(|f| render(f, &app))
        .expect("Terminal draw failed for populated state");

    let output = buffer_to_string(&terminal);

    assert!(output.contains("[User]"));
    assert!(output.contains("Generate an architecture design"));
    assert!(output.contains("[LDrago - Coding]"));
    assert!(output.contains("Architecture draft: Modular pipeline"));
    assert!(output.contains("64.2 tokens/sec"));
    assert!(output.contains("4 snapshots"));
    assert!(output.contains("plan"));
    assert!(output.contains("Chat [Active]"));
}

#[test]
fn test_render_to_terminal_helper() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create TestBackend terminal");
    let app = AppState::new();

    let result = render_to_terminal(&mut terminal, &app);
    assert!(result.is_ok(), "render_to_terminal must succeed");

    let output = buffer_to_string(&terminal);
    assert!(output.contains("LDRAGO"));
    assert!(output.contains("Engines [Active]")); // Default pane is Engines
}

#[test]
fn test_cycle_brain_and_render_all_brains() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    let mut app = AppState::new();

    // 1. Initial brain: Coding
    assert_eq!(app.selected_brain, BrainType::Coding);
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("> [1] Coding Brain"));

    // 2. Cycle to Design
    app.cycle_brain();
    assert_eq!(app.selected_brain, BrainType::Design);
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("> [2] Design Brain"));

    // 3. Cycle to General
    app.cycle_brain();
    assert_eq!(app.selected_brain, BrainType::General);
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("> [3] General Brain"));

    // 4. Cycle back to Coding
    app.cycle_brain();
    assert_eq!(app.selected_brain, BrainType::Coding);
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("> [1] Coding Brain"));
}

#[test]
fn test_active_pane_highlighting() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    let mut app = AppState::new();

    // Engines active
    app.active_pane = ActivePane::Engines;
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("Engines [Active]"));

    // Chat active
    app.active_pane = ActivePane::Chat;
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("Chat [Active]"));

    // Telemetry active
    app.active_pane = ActivePane::Telemetry;
    terminal.draw(|f| render(f, &app)).unwrap();
    let out = buffer_to_string(&terminal);
    assert!(out.contains("Telemetry & Safety [Active]"));
}

#[test]
fn test_scroll_operations_no_panic() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    let mut app = AppState::new();

    for i in 0..50 {
        app.add_message("User", &format!("User question #{i}"));
        app.add_message("Assistant", &format!("Assistant reply #{i}\nWith multiple lines"));
    }

    // Scroll up several times
    for _ in 0..10 {
        app.scroll_up();
    }
    assert_eq!(app.scroll_offset, 10);
    terminal.draw(|f| render(f, &app)).unwrap();

    // Scroll down
    for _ in 0..5 {
        app.scroll_down();
    }
    assert_eq!(app.scroll_offset, 5);
    terminal.draw(|f| render(f, &app)).unwrap();

    // Scroll down beyond zero (saturates at 0)
    for _ in 0..10 {
        app.scroll_down();
    }
    assert_eq!(app.scroll_offset, 0);
    terminal.draw(|f| render(f, &app)).unwrap();
}
