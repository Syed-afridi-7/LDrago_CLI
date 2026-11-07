//! Model switching with context preservation and provenance tracking.

use chrono::Utc;
use ld_schemas::{BrainType, Role, Turn};
use serde::{Deserialize, Serialize};

/// Provenance record tracking every model switch during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchProvenance {
    pub from_brain: BrainType,
    pub to_brain: BrainType,
    pub from_model: String,
    pub to_model: String,
    pub reason: String,
    pub timestamp: String,
    pub context_turns_preserved: usize,
    pub user_confirmed: bool,
}

/// Context container preserving state across model switches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelSwitchContext {
    pub turns: Vec<Turn>,
    pub switch_history: Vec<SwitchProvenance>,
}

impl ModelSwitchContext {
    pub fn new() -> Self {
        Self {
            turns: Vec::new(),
            switch_history: Vec::new(),
        }
    }

    /// Add a turn to the shared conversation history.
    pub fn add_turn(&mut self, turn: Turn) {
        self.turns.push(turn);
    }

    /// Record a model switch event.
    pub fn record_switch(
        &mut self,
        from_brain: BrainType,
        to_brain: BrainType,
        from_model: impl Into<String>,
        to_model: impl Into<String>,
        reason: impl Into<String>,
        user_confirmed: bool,
    ) {
        let provenance = SwitchProvenance {
            from_brain,
            to_brain,
            from_model: from_model.into(),
            to_model: to_model.into(),
            reason: reason.into(),
            timestamp: Utc::now().to_rfc3339(),
            context_turns_preserved: self.turns.len(),
            user_confirmed,
        };
        self.switch_history.push(provenance);
    }

    /// Format current conversation turns as a consolidated prompt for the target brain.
    pub fn format_prompt_for_target(&self, system_instruction: Option<&str>, current_prompt: &str) -> String {
        let mut full_prompt = String::new();

        if let Some(sys) = system_instruction {
            full_prompt.push_str(&format!("System: {sys}\n\n"));
        }

        for turn in &self.turns {
            match turn.role {
                Role::System => full_prompt.push_str(&format!("System: {}\n", turn.content)),
                Role::User => full_prompt.push_str(&format!("User: {}\n", turn.content)),
                Role::Assistant => full_prompt.push_str(&format!("Assistant: {}\n", turn.content)),
                Role::Tool => full_prompt.push_str(&format!("Tool: {}\n", turn.content)),
            }
        }

        full_prompt.push_str(&format!("User: {current_prompt}\nAssistant:"));
        full_prompt
    }
}
