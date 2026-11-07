//! Application state and event handling for the LDrago interactive TUI.

use ld_schemas::BrainType;

/// The currently focused/active pane in the terminal interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Engines,
    Chat,
    Telemetry,
}

impl Default for ActivePane {
    fn default() -> Self {
        Self::Engines
    }
}

/// Central state model for the interactive TUI session.
#[derive(Debug, Clone)]
pub struct AppState {
    pub active_pane: ActivePane,
    pub input: String,
    pub messages: Vec<(String, String)>,
    pub selected_brain: BrainType,
    pub coder_healthy: bool,
    pub designer_healthy: bool,
    pub ollama_healthy: bool,
    pub tokens_per_sec: f64,
    pub active_checkpoints: usize,
    pub scroll_offset: usize,
    pub running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Initialize a new TUI application state with defaults.
    pub fn new() -> Self {
        Self {
            active_pane: ActivePane::Engines,
            input: String::new(),
            messages: Vec::new(),
            selected_brain: BrainType::Coding,
            coder_healthy: false,
            designer_healthy: false,
            ollama_healthy: false,
            tokens_per_sec: 0.0,
            active_checkpoints: 0,
            scroll_offset: 0,
            running: true,
        }
    }

    /// Cycle active pane focus: Engines -> Chat -> Telemetry -> Engines.
    pub fn cycle_focus(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::Engines => ActivePane::Chat,
            ActivePane::Chat => ActivePane::Telemetry,
            ActivePane::Telemetry => ActivePane::Engines,
        };
    }

    /// Cycle selected brain: Coding -> Design -> General -> Coding.
    pub fn cycle_brain(&mut self) {
        self.selected_brain = match self.selected_brain {
            BrainType::Coding => BrainType::Design,
            BrainType::Design => BrainType::General,
            BrainType::General => BrainType::Coding,
        };
    }

    /// Select a specific brain directly.
    pub fn set_brain(&mut self, brain: BrainType) {
        self.selected_brain = brain;
    }

    /// Scroll up (reveal older messages in history).
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll down (reveal newer messages in history).
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Append a character to the active user prompt input buffer.
    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    /// Remove the trailing character from the prompt input buffer.
    pub fn delete_char(&mut self) {
        self.input.pop();
    }

    /// Submit current input buffer. Returns Some(text) if non-empty, and clears input buffer.
    pub fn submit_input(&mut self) -> Option<String> {
        let trimmed = self.input.trim().to_string();
        self.input.clear();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }

    /// Update health probe statuses across all 3 AI engine backends.
    pub fn set_health(&mut self, coder: bool, designer: bool, ollama: bool) {
        self.coder_healthy = coder;
        self.designer_healthy = designer;
        self.ollama_healthy = ollama;
    }

    /// Add a completed conversational turn to the message history.
    pub fn add_message(&mut self, sender: &str, text: &str) {
        self.messages.push((sender.to_string(), text.to_string()));
    }

    /// Append a streaming chunk of tokens to the most recent message.
    pub fn append_to_last_message(&mut self, chunk: &str) {
        if let Some((_, content)) = self.messages.last_mut() {
            content.push_str(chunk);
        }
    }

    /// Signal the event loop to gracefully shut down the TUI.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
