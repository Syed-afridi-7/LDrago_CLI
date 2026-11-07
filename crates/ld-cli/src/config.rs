//! Layered configuration management for LDrago CLI via Figment and TOML.

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub coding_endpoint: String,
    pub design_endpoint: String,
    pub ollama_endpoint: String,
    pub ollama_model: String,
    pub default_brain: String,
    pub request_timeout_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            coding_endpoint: "http://127.0.0.1:8741".to_string(),
            design_endpoint: "http://127.0.0.1:8741".to_string(),
            ollama_endpoint: "http://127.0.0.1:11434".to_string(),
            ollama_model: "llama3".to_string(),
            default_brain: "coding".to_string(),
            request_timeout_secs: 30,
        }
    }
}

impl AppConfig {
    /// Load configuration following precedence:
    /// CLI overrides > Env vars (LDRAGO_*) > Project .ldrago.toml > Default fallback
    pub fn load(custom_path: Option<PathBuf>) -> Result<Self, figment::Error> {
        let mut figment = Figment::from(Serialized::defaults(AppConfig::default()));

        if let Some(path) = custom_path {
            figment = figment.merge(Toml::file(path));
        } else if std::path::Path::new(".ldrago.toml").exists() {
            figment = figment.merge(Toml::file(".ldrago.toml"));
        }

        figment = figment.merge(Env::prefixed("LDRAGO_"));

        figment.extract()
    }
}
