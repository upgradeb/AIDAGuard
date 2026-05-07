use tracing::warn;

use crate::client::UpstreamClient;
use crate::provider::ProviderRegistry;
use crate::types::{ProviderConfig, UpstreamConfig};

/// High-level manager for upstream providers.
///
/// Holds a registry of known providers (from built-in YAML definitions)
/// and user-configured upstream connections. Resolves target URLs to
/// matching upstreams and creates configured clients.
#[derive(Debug, Default)]
pub struct UpstreamManager {
    registry: ProviderRegistry,
    upstreams: Vec<UpstreamConfig>,
}

impl UpstreamManager {
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
            upstreams: Vec::new(),
        }
    }

    /// Load all built-in provider definitions (embedded at compile time).
    pub fn load_builtins(&mut self) -> usize {
        let builtins: &[&str] = &[
            include_str!("providers/openai.yaml"),
            include_str!("providers/anthropic.yaml"),
            include_str!("providers/deepseek.yaml"),
            include_str!("providers/qwen.yaml"),
            include_str!("providers/zhipu.yaml"),
            include_str!("providers/groq.yaml"),
            include_str!("providers/gemini.yaml"),
        ];

        let mut count = 0;
        for yaml in builtins {
            match serde_yaml::from_str::<ProviderConfig>(yaml) {
                Ok(config) => {
                    self.registry.register(config);
                    count += 1;
                }
                Err(e) => {
                    warn!("Failed to parse built-in provider YAML: {}", e);
                }
            }
        }
        count
    }

    /// Set user-configured upstreams (from config.toml or settings).
    pub fn set_upstreams(&mut self, upstreams: Vec<UpstreamConfig>) {
        self.upstreams = upstreams;
    }

    /// Access the provider registry.
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// List all user-configured upstreams.
    pub fn upstreams(&self) -> &[UpstreamConfig] {
        &self.upstreams
    }

    /// Resolve a target URL to find the matching user upstream config.
    ///
    /// Returns the upstream whose provider endpoint is a prefix of the target URL.
    pub fn resolve(&self, target_url: &str) -> Option<&UpstreamConfig> {
        self.upstreams.iter().find(|u| {
            let endpoint = &u.provider.endpoint;
            target_url.starts_with(endpoint)
        })
    }

    /// Find the built-in provider config for a given provider id.
    pub fn find_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.registry.get(id)
    }

    /// Match a target URL against built-in provider endpoints.
    ///
    /// Returns the provider whose endpoint is a prefix of the target URL.
    pub fn find_by_endpoint(&self, target_url: &str) -> Option<&ProviderConfig> {
        self.registry.iter().find(|p| target_url.starts_with(&p.endpoint))
    }

    /// Create an UpstreamClient from a user-configured upstream.
    ///
    /// Returns None if the provider is not found in the built-in registry.
    pub fn create_client(&self, upstream: &UpstreamConfig) -> Option<UpstreamClient> {
        let api_key = upstream.api_key.as_deref().unwrap_or("");
        UpstreamClient::new(
            upstream.provider.clone(),
            api_key.to_string(),
            upstream.timeout_secs,
        )
        .ok()
    }
}
