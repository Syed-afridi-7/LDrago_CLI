//! Capability-based model router for LDrago engines.

use std::sync::Arc;
use ld_schemas::{BrainType, HealthStatus, ModelCapabilities};
use crate::adapter::{ModelAdapter, ModelError};

/// Router managing the three AI engines.
#[derive(Clone, Default)]
pub struct ModelRouter {
    adapters: Vec<Arc<dyn ModelAdapter>>,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    /// Register a model adapter.
    pub fn register(&mut self, adapter: Arc<dyn ModelAdapter>) {
        self.adapters.push(adapter);
    }

    /// List all registered model capabilities.
    pub fn list_capabilities(&self) -> Vec<ModelCapabilities> {
        self.adapters.iter().map(|a| a.capabilities()).collect()
    }

    /// Route to the appropriate adapter based on requested BrainType and health status.
    pub async fn route(&self, brain: BrainType) -> Result<Arc<dyn ModelAdapter>, ModelError> {
        // 1. Direct match for requested BrainType
        for adapter in &self.adapters {
            if adapter.capabilities().brain == Some(brain) {
                if let Ok(HealthStatus::Healthy) = adapter.health_check().await {
                    return Ok(Arc::clone(adapter));
                }
            }
        }

        // 2. If preferred brain is degraded or unreachable, check for General/Ollama fallback
        for adapter in &self.adapters {
            if adapter.capabilities().brain == Some(BrainType::General) {
                if let Ok(HealthStatus::Healthy) = adapter.health_check().await {
                    return Ok(Arc::clone(adapter));
                }
            }
        }

        // 3. If any adapter matches the brain regardless of health status, return it
        for adapter in &self.adapters {
            if adapter.capabilities().brain == Some(brain) {
                return Ok(Arc::clone(adapter));
            }
        }

        // 4. Return first registered adapter or fail
        self.adapters
            .first()
            .map(Arc::clone)
            .ok_or_else(|| ModelError::Adapter("No model engines registered in router".to_string()))
    }
}
