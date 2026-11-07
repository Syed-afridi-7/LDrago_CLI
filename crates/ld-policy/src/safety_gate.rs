use crate::rules::RiskLevel;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyVerdict {
    Safe,
    Violation {
        reason: String,
        risk: RiskLevel,
    },
}

#[derive(Debug, Clone)]
pub struct SafetyGate {
    injection_patterns: Vec<Regex>,
    destructive_command_patterns: Vec<Regex>,
    credential_patterns: Vec<Regex>,
}

impl Default for SafetyGate {
    fn default() -> Self {
        let injection_patterns = vec![
            Regex::new(r"(?i)\bignore\s+(all\s+)?previous\s+instructions\b").unwrap(),
            Regex::new(r"(?i)\bsystem\s+prompt\s+override\b").unwrap(),
            Regex::new(r"(?i)\byou\s+are\s+now\s+in\s+developer\s+mode\b").unwrap(),
            Regex::new(r"(?i)\bdisregard\s+all\s+(rules|guidelines)\b").unwrap(),
        ];

        let destructive_command_patterns = vec![
            Regex::new(r"(?i)\brm\s+(-[rfRF]{1,4}\s+)?(/\s*|\.\s*|/\*)").unwrap(),
            Regex::new(r"(?i)\bmkfs(\.\w+)?\b").unwrap(),
            Regex::new(r"(?i)\bdd\s+if=").unwrap(),
            Regex::new(r"(?i):\(\)\s*\{\s*:\|:&\s*\};:").unwrap(),
            Regex::new(r"(?i)\bformat\s+[a-z]:").unwrap(),
            Regex::new(r"(?i)\bdrop\s+database\b").unwrap(),
            Regex::new(r"(?i)\bdrop\s+table\b").unwrap(),
            Regex::new(r"(?i)\bchmod\s+(-R\s+)?777\s+/").unwrap(),
        ];

        let credential_patterns = vec![
            Regex::new(r"(?i)\bAKIA[0-9A-Z]{16}\b").unwrap(), // AWS Access Key
            Regex::new(r"(?i)bearer\s+ey[a-zA-Z0-9_-]{20,}").unwrap(), // JWT
            Regex::new(r"(?i)-----BEGIN\s+(RSA|EC|OPENSSH|PGP)?\s*PRIVATE\s+KEY-----").unwrap(),
        ];

        Self {
            injection_patterns,
            destructive_command_patterns,
            credential_patterns,
        }
    }
}

impl SafetyGate {
    pub fn check_prompt(&self, prompt: &str) -> SafetyVerdict {
        for pattern in &self.injection_patterns {
            if pattern.is_match(prompt) {
                return SafetyVerdict::Violation {
                    reason: "Detected prompt injection / jailbreak attempt".to_string(),
                    risk: RiskLevel::High,
                };
            }
        }

        for pattern in &self.credential_patterns {
            if pattern.is_match(prompt) {
                return SafetyVerdict::Violation {
                    reason: "Detected raw credential / secret pattern in prompt".to_string(),
                    risk: RiskLevel::Dangerous,
                };
            }
        }

        SafetyVerdict::Safe
    }

    pub fn check_command(&self, command: &str) -> SafetyVerdict {
        for pattern in &self.destructive_command_patterns {
            if pattern.is_match(command) {
                return SafetyVerdict::Violation {
                    reason: format!("Destructive command pattern blocked: {}", command),
                    risk: RiskLevel::Dangerous,
                };
            }
        }

        SafetyVerdict::Safe
    }
}
