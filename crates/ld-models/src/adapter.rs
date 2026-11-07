//! Model adapter trait definitions and common error types.

use async_trait::async_trait;
use ld_schemas::{CompletionRequest, CompletionResponse, HealthStatus, ModelCapabilities};

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Adapter error: {0}")]
    Adapter(String),
}

/// Unified asynchronous model interface implemented by all brains and backends.
#[async_trait]
pub trait ModelAdapter: Send + Sync {
    /// Perform a health check probe against the model runtime.
    async fn health_check(&self) -> Result<HealthStatus, ModelError>;

    /// Generate text completion based on a structured request.
    async fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, ModelError>;

    /// Retrieve model capabilities and security metadata.
    fn capabilities(&self) -> ModelCapabilities;
}
