//! Adapter for custom PyTorch neural networks hosted on local FastAPI runtime.

use async_trait::async_trait;
use ld_schemas::{
    BrainType, CompletionRequest, CompletionResponse, HealthStatus, ModelCapabilities, PrivacyClass,
};
use reqwest::Client;
use std::time::Duration;

use crate::adapter::{ModelAdapter, ModelError};

/// Client for communicating with the local Python FastAPI model server.
#[derive(Clone, Debug)]
pub struct CustomBrainAdapter {
    client: Client,
    base_url: String,
    #[allow(dead_code)]
    brain: BrainType,
    capabilities: ModelCapabilities,
}

impl CustomBrainAdapter {
    pub fn new(base_url: impl Into<String>, brain: BrainType) -> Self {
        let base_url = base_url.into();
        let alias = match brain {
            BrainType::Coding => "coder-local".to_string(),
            BrainType::Design => "designer-local".to_string(),
            BrainType::General => "general-local".to_string(),
        };

        let model_name = match brain {
            BrainType::Coding => "ldrago-coder-custom-transformer".to_string(),
            BrainType::Design => "ldrago-designer-custom-ssm".to_string(),
            BrainType::General => "ldrago-general".to_string(),
        };

        let caps = match brain {
            BrainType::Coding => vec![
                "coding".to_string(),
                "reasoning".to_string(),
                "tool_calling".to_string(),
            ],
            BrainType::Design => vec!["design".to_string(), "reasoning".to_string()],
            BrainType::General => vec!["general".to_string()],
        };

        let capabilities = ModelCapabilities {
            alias,
            endpoint: format!("{base_url}/v1"),
            model_name,
            capabilities: caps,
            context_window: 2048,
            privacy: PrivacyClass::Local,
            brain: Some(brain),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url,
            brain,
            capabilities,
        }
    }
}

#[async_trait]
impl ModelAdapter for CustomBrainAdapter {
    async fn health_check(&self) -> Result<HealthStatus, ModelError> {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => Ok(HealthStatus::Healthy),
            Ok(_resp) => Ok(HealthStatus::Degraded),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }

    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ModelError> {
        let url = format!("{}/v1/complete", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| ModelError::Adapter(format!("HTTP error from model server: {e}")))?;

        let completion = resp.json::<CompletionResponse>().await?;
        Ok(completion)
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities.clone()
    }
}
