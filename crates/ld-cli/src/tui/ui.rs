//! Responsive terminal UI layout and widget rendering for LDrago CLI.

use crate::tui::app::{ActivePane, AppState};
use ld_schemas::BrainType;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Gold accent color matching LDrago dragon branding.
pub const LDRAGO_GOLD: Color = Color::Rgb(255, 215, 0);

/// Dragon crimson red accent color.
pub const LDRAGO_RED: Color = Color::Rgb(255, 68, 68);

/// Cyan accent color for user and system highlights.
pub const LDRAGO_CYAN: Color = Color::Cyan;

/// Render the complete 3-pane responsive terminal layout.
pub fn render(frame: &mut Frame, app: &AppState) {
    let area = frame.area();

    // Outer layout: Header (Length 3), Main 3-column body (Min 10), Footer (Length 1)
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main 3-pane body
            Constraint::Length(1), // Footer keyboard instructions
        ])
        .split(area);

    render_header(frame, outer_chunks[0], app);
    render_main_body(frame, outer_chunks[1], app);
    render_footer(frame, outer_chunks[2]);
}

/// Render the top title header banner with TrueColor styling.
fn render_header(frame: &mut Frame, area: Rect, app: &AppState) {
    let brain_label = match app.selected_brain {
        BrainType::Coding => "CODING (Transformer)",
        BrainType::Design => "DESIGN (SSM Mamba)",
        BrainType::General => "GENERAL (Ollama)",
    };

    let title_line = Line::from(vec![
        Span::styled(
            "🐉 LDRAGO AI — LOCAL TWIN TERMINAL",
            Style::default().fg(LDRAGO_GOLD).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            format!("| ACTIVE BRAIN: [{brain_label}]"),
            Style::default().fg(LDRAGO_CYAN).bold(),
        ),
    ]);

    let header_widget = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(LDRAGO_GOLD)),
    );

    frame.render_widget(header_widget, area);
}

/// Render the 3-column main body (Engines | Chat & Input | Telemetry & Safety).
fn render_main_body(frame: &mut Frame, area: Rect, app: &AppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Engines panel
            Constraint::Percentage(50), // Chat & Input panel
            Constraint::Percentage(25), // Telemetry & Safety panel
        ])
        .split(area);

    render_engines_panel(frame, main_chunks[0], app);
    render_chat_panel(frame, main_chunks[1], app);
    render_telemetry_panel(frame, main_chunks[2], app);
}

/// Render the left AI Engines selection & health probe panel.
fn render_engines_panel(frame: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.active_pane == ActivePane::Engines;
    let border_color = if is_focused {
        LDRAGO_GOLD
    } else {
        Color::DarkGray
    };

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "REGISTERED BRAINS",
        Style::default().fg(LDRAGO_CYAN).bold(),
    )));
    lines.push(Line::from(""));

    // 1. Coding Brain: PyTorch Transformer (🟢 / 🔴)
    let coder_sel = app.selected_brain == BrainType::Coding;
    let coder_prefix = if coder_sel { "> " } else { "  " };
    let coder_style = if coder_sel {
        Style::default().fg(LDRAGO_GOLD).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let coder_indicator = if app.coder_healthy { "🟢" } else { "🔴" };
    lines.push(Line::from(vec![
        Span::styled(coder_prefix, coder_style),
        Span::styled("[1] Coding Brain ", coder_style),
        Span::raw(coder_indicator),
    ]));
    lines.push(Line::from(Span::styled(
        "    PyTorch Transformer",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));

    // 2. Design Brain: PyTorch SSM Mamba (🟢 / 🔴)
    let designer_sel = app.selected_brain == BrainType::Design;
    let designer_prefix = if designer_sel { "> " } else { "  " };
    let designer_style = if designer_sel {
        Style::default().fg(LDRAGO_GOLD).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let designer_indicator = if app.designer_healthy { "🟢" } else { "🔴" };
    lines.push(Line::from(vec![
        Span::styled(designer_prefix, designer_style),
        Span::styled("[2] Design Brain ", designer_style),
        Span::raw(designer_indicator),
    ]));
    lines.push(Line::from(Span::styled(
        "    PyTorch SSM Mamba",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(""));

    // 3. General Brain: Ollama Adapter (🟢 / ⚪)
    let general_sel = app.selected_brain == BrainType::General;
    let general_prefix = if general_sel { "> " } else { "  " };
    let general_style = if general_sel {
        Style::default().fg(LDRAGO_GOLD).bold()
    } else {
        Style::default().fg(Color::White)
    };
    let general_indicator = if app.ollama_healthy { "🟢" } else { "⚪" };
    lines.push(Line::from(vec![
        Span::styled(general_prefix, general_style),
        Span::styled("[3] General Brain ", general_style),
        Span::raw(general_indicator),
    ]));
    lines.push(Line::from(Span::styled(
        "    Ollama Adapter",
        Style::default().fg(Color::Gray),
    )));

    let panel = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(if is_focused {
                " Engines [Active] "
            } else {
                " Engines "
            })
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(panel, area);
}

/// Render the center Chat message stream and input buffer panel.
fn render_chat_panel(frame: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.active_pane == ActivePane::Chat;
    let border_color = if is_focused {
        LDRAGO_GOLD
    } else {
        Color::DarkGray
    };

    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Message Area
            Constraint::Length(3), // Input Box
        ])
        .split(area);

    // 1. Message Area
    let mut message_lines = Vec::new();
    if app.messages.is_empty() {
        message_lines.push(Line::from(Span::styled(
            "Welcome to LDrago Local Twin Terminal.",
            Style::default().fg(Color::Gray),
        )));
        message_lines.push(Line::from(Span::styled(
            "Type a prompt below and press Enter to chat.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (sender, text) in &app.messages {
            let is_user = sender.eq_ignore_ascii_case("user") || sender.starts_with("[User]");
            let tag = if is_user {
                Span::styled("[User] ", Style::default().fg(LDRAGO_CYAN).bold())
            } else {
                let tag_str = match app.selected_brain {
                    BrainType::Coding => "[LDrago - Coding] ",
                    BrainType::Design => "[LDrago - Design] ",
                    BrainType::General => "[LDrago - General] ",
                };
                Span::styled(tag_str, Style::default().fg(LDRAGO_GOLD).bold())
            };

            let mut first = true;
            for line in text.lines() {
                if first {
                    message_lines.push(Line::from(vec![
                        tag.clone(),
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]));
                    first = false;
                } else {
                    message_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]));
                }
            }
            if first {
                message_lines.push(Line::from(vec![tag.clone()]));
            }
            message_lines.push(Line::from(""));
        }
    }

    let total_lines = u16::try_from(message_lines.len()).unwrap_or(u16::MAX);
    let scroll_offset = u16::try_from(app.scroll_offset).unwrap_or(u16::MAX);
    let visible_height = chat_chunks[0].height.saturating_sub(2);
    let scroll_y = if total_lines > visible_height {
        (total_lines - visible_height).saturating_sub(scroll_offset)
    } else {
        0
    };

    let messages_widget = Paragraph::new(message_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if is_focused {
                    " Chat [Active] "
                } else {
                    " Chat "
                })
                .border_style(Style::default().fg(border_color)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));

    frame.render_widget(messages_widget, chat_chunks[0]);

    // 2. Input Box
    let input_line = Line::from(vec![
        Span::styled("❯ ", Style::default().fg(LDRAGO_GOLD).bold()),
        Span::styled(&app.input, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(LDRAGO_GOLD)),
    ]);

    let input_widget = Paragraph::new(input_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Input ")
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(input_widget, chat_chunks[1]);
}

/// Render the right Telemetry & Policy Safety status panel.
fn render_telemetry_panel(frame: &mut Frame, area: Rect, app: &AppState) {
    let is_focused = app.active_pane == ActivePane::Telemetry;
    let border_color = if is_focused {
        LDRAGO_GOLD
    } else {
        Color::DarkGray
    };

    let lines = vec![
        Line::from(Span::styled(
            "SYSTEM STATUS",
            Style::default().fg(LDRAGO_CYAN).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled("Inference Speed:", Style::default().bold())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{:.1} tokens/sec", app.tokens_per_sec),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Active Checkpoints:",
            Style::default().bold(),
        )),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} snapshots", app.active_checkpoints),
                Style::default().fg(LDRAGO_GOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "SECURITY & SAFETY",
            Style::default().fg(LDRAGO_CYAN).bold(),
        )),
        Line::from(""),
        Line::from(Span::styled("Policy Engine:", Style::default().bold())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("ACTIVE (Zero-Trust)", Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Sandbox:", Style::default().bold())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "Path Containment Enabled",
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let panel = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(if is_focused {
                " Telemetry & Safety [Active] "
            } else {
                " Telemetry & Safety "
            })
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(panel, area);
}

/// Render the bottom footer bar with keyboard shortcuts.
fn render_footer(frame: &mut Frame, area: Rect) {
    let instructions = Line::from(vec![
        Span::styled(" [Tab] ", Style::default().fg(LDRAGO_GOLD).bold()),
        Span::styled("Switch Pane | ", Style::default().fg(Color::Gray)),
        Span::styled("[1-3 / Space] ", Style::default().fg(LDRAGO_GOLD).bold()),
        Span::styled("Cycle Brain | ", Style::default().fg(Color::Gray)),
        Span::styled("[Enter] ", Style::default().fg(LDRAGO_GOLD).bold()),
        Span::styled("Send | ", Style::default().fg(Color::Gray)),
        Span::styled("[Esc] ", Style::default().fg(LDRAGO_GOLD).bold()),
        Span::styled("Quit", Style::default().fg(Color::Gray)),
    ]);

    let footer = Paragraph::new(instructions);
    frame.render_widget(footer, area);
}
