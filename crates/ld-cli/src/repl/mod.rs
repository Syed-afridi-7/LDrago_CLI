//! Interactive REPL session management, state machine, and slash commands.

pub mod banner;
pub mod commands;

pub use banner::print_session_banner;
pub use commands::{parse_repl_command, ReplCommand};

use ld_schemas::BrainType;

/// Interactive REPL session state.
#[derive(Debug, Clone)]
pub struct ReplSession {
    pub active_brain: BrainType,
    pub running: bool,
}

impl Default for ReplSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplSession {
    /// Create a new REPL session defaulting to the Coding brain.
    pub fn new() -> Self {
        Self {
            active_brain: BrainType::Coding,
            running: true,
        }
    }

    /// Cycle to the next model in sequence: Coding -> Design -> General -> Coding.
    pub fn cycle_model(&mut self) -> BrainType {
        self.active_brain = match self.active_brain {
            BrainType::Coding => BrainType::Design,
            BrainType::Design => BrainType::General,
            BrainType::General => BrainType::Coding,
        };
        self.active_brain
    }

    /// Set the active brain by alias or number (case-insensitive).
    /// Returns true if recognized and set, false otherwise.
    pub fn set_model(&mut self, name: &str) -> bool {
        match name.trim().to_lowercase().as_str() {
            "coding" | "coder" | "1" => {
                self.active_brain = BrainType::Coding;
                true
            }
            "design" | "designer" | "mamba" | "2" => {
                self.active_brain = BrainType::Design;
                true
            }
            "general" | "ollama" | "3" => {
                self.active_brain = BrainType::General;
                true
            }
            _ => false,
        }
    }

    /// Return prompt prefix for current active brain (e.g. "ldrago (coding) > ").
    pub fn prompt_prefix(&self) -> String {
        let label = match self.active_brain {
            BrainType::Coding => "coding",
            BrainType::Design => "design",
            BrainType::General => "general",
        };
        format!("ldrago ({label}) > ")
    }

    /// Parse user input line against this session's commands.
    pub fn parse_input(&self, input: &str) -> ReplCommand {
        commands::parse_repl_command(input)
    }
}
