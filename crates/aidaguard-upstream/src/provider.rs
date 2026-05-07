use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::types::ProviderConfig;

/// Registry of LLM providers loaded from YAML files
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, ProviderConfig>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Load all YAML provider files from a directory
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read provider directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                let contents = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read provider file: {}", path.display()))?;
                let config: ProviderConfig = serde_yaml::from_str(&contents)
                    .with_context(|| format!("YAML parse failed: {}", path.display()))?;
                self.providers.insert(config.id.clone(), config);
            }
        }
        Ok(self.providers.len())
    }

    /// Register a built-in provider
    pub fn register(&mut self, config: ProviderConfig) {
        self.providers.insert(config.id.clone(), config);
    }

    /// Get a provider by id
    pub fn get(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    /// List all registered provider ids
    pub fn list_ids(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered providers
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Iterate over all providers
    pub fn iter(&self) -> impl Iterator<Item = &ProviderConfig> {
        self.providers.values()
    }
}
