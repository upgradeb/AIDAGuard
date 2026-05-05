use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::ToolAdapter;

/// Static metadata for a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub config_path_template: String,
    pub categories: Vec<String>,
}

/// Extends ToolAdapter with plugin metadata.
pub trait Plugin: ToolAdapter {
    fn manifest(&self) -> PluginManifest;
}

/// Per-plugin runtime state persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginStateEntry {
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginStateFile {
    plugins: HashMap<String, PluginStateEntry>,
}

/// Central registry for all tool plugins. Stored in Tauri AppState.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
    /// enabled state per plugin id
    states: HashMap<String, bool>,
    /// path to plugins.json state file
    state_path: PathBuf,
}

impl PluginRegistry {
    pub fn new(state_dir: PathBuf) -> Self {
        let state_path = state_dir.join("plugins.json");
        let mut registry = Self {
            plugins: Vec::new(),
            states: HashMap::new(),
            state_path,
        };
        registry.load_state();
        registry
    }

    /// Register a plugin. All built-in plugins are registered at startup via this method.
    /// New plugins default to enabled=true.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let id = plugin.id().to_string();
        // Default to enabled if not already in state
        self.states.entry(id.clone()).or_insert(true);
        self.plugins.push(plugin);
    }

    /// Return all registered plugin manifests with their enabled state.
    pub fn all_manifests(&self) -> Vec<(PluginManifest, bool)> {
        self.plugins.iter().map(|p| {
            let enabled = self.states.get(p.id()).copied().unwrap_or(true);
            (p.manifest(), enabled)
        }).collect()
    }

    /// Find a plugin adapter by id (respects enabled state).
    pub fn get(&self, id: &str) -> Option<&dyn Plugin> {
        self.plugins.iter().find(|p| p.id() == id).map(|p| &**p)
    }

    /// Enable a plugin by id. Persists state.
    pub fn enable(&mut self, id: &str) -> Result<(), String> {
        if !self.plugins.iter().any(|p| p.id() == id) {
            return Err(format!("Plugin {} not found", id));
        }
        self.states.insert(id.to_string(), true);
        self.save_state()
    }

    /// Disable a plugin by id. Persists state.
    pub fn disable(&mut self, id: &str) -> Result<(), String> {
        if !self.plugins.iter().any(|p| p.id() == id) {
            return Err(format!("Plugin {} not found", id));
        }
        self.states.insert(id.to_string(), false);
        self.save_state()
    }

    /// Check if a plugin is enabled.
    pub fn is_enabled(&self, id: &str) -> bool {
        self.states.get(id).copied().unwrap_or(true)
    }

    /// Iterate over all registered plugins (regardless of enabled state).
    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Plugin>> {
        self.plugins.iter()
    }

    /// Iterate over enabled plugins only.
    pub fn iter_enabled(&self) -> impl Iterator<Item = &Box<dyn Plugin>> {
        self.plugins.iter().filter(|p| self.is_enabled(p.id()))
    }

    fn load_state(&mut self) {
        if let Ok(content) = std::fs::read_to_string(&self.state_path) {
            if let Ok(file) = serde_json::from_str::<PluginStateFile>(&content) {
                for (id, entry) in file.plugins {
                    self.states.insert(id, entry.enabled);
                }
            }
        }
    }

    fn save_state(&self) -> Result<(), String> {
        if let Some(parent) = self.state_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let file = PluginStateFile {
            plugins: self.states.iter().map(|(k, v)| {
                (k.clone(), PluginStateEntry { enabled: *v })
            }).collect(),
        };
        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize plugin state: {}", e))?;
        std::fs::write(&self.state_path, content)
            .map_err(|e| format!("Failed to write plugin state: {}", e))?;
        Ok(())
    }
}
