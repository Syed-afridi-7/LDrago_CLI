//! Adapter for Ollama local inference engine.

use async_trait::async_trait;
use ld_schemas::{
    BrainType, CompletionRequest, CompletionResponse, HealthStatus, ModelCapabilities, PrivacyClass,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::adapter::{ModelAdapter, ModelError};

/// Client for communicating with the local Ollama daemon.
#[derive(Clone, Debug)]
pub struct OllamaAdapter {
    client: Client,
    base_url: String,
    model_name: String,
    capabilities: ModelCapabilities,
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

impl OllamaAdapter {
    pub fn new(base_url: impl Into<String>, model_name: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let model_name = model_name.into();

        let capabilities = ModelCapabilities {
            alias: "ollama-general".to_string(),
            endpoint: format!("{base_url}/api"),
            model_name: model_name.clone(),
            capabilities: vec![
                "coding".to_string(),
                "design".to_string(),
                "reasoning".to_string(),
            ],
            context_window: 8192,
            privacy: PrivacyClass::Local,
            brain: Some(BrainType::General),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url,
            model_name,
            capabilities,
        }
    }
}

#[async_trait]
impl ModelAdapter for OllamaAdapter {
    async fn health_check(&self) -> Result<HealthStatus, ModelError> {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(HealthStatus::Healthy),
            Ok(_) => Ok(HealthStatus::Degraded),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}/api/generate", self.base_url);
        let payload = OllamaGenerateRequest {
            model: &self.model_name,
            prompt: &req.prompt,
            stream: false,
        };

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| ModelError::Adapter(format!("Ollama API error: {e}")))?;

        let ollama_resp = resp.json::<OllamaGenerateResponse>().await?;

        Ok(CompletionResponse {
            text: ollama_resp.response,
            brain: req.brain,
            tokens_generated: None,
        })
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities.clone()
    }
}
