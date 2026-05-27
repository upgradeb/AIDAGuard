use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};
use super::manifest::*;
use super::json_path;

/// A generic tool adapter driven by a JSON manifest.
pub struct DeclarativeAdapter {
    manifest: ToolManifest,
}

impl DeclarativeAdapter {
    pub fn new(manifest: ToolManifest) -> Self {
        Self { manifest }
    }

    /// Expand `~` in a path to the user's home directory.
    fn expand_path(path: &str) -> Option<PathBuf> {
        if path.starts_with("~/") {
            home_dir().map(|h| h.join(&path[2..]))
        } else if path == "~" {
            home_dir()
        } else {
            Some(PathBuf::from(path))
        }
    }

    /// Collect all config files (primary + secondary) for iteration.
    fn all_configs(&self) -> Vec<&FileConfig> {
        let mut cfgs: Vec<&FileConfig> = Vec::new();
        if let Some(ref cfg) = self.manifest.config {
            cfgs.push(cfg);
        }
        for cfg in &self.manifest.secondary_configs {
            cfgs.push(cfg);
        }
        cfgs
    }

    /// Read a config file into a serde_json::Value.
    fn read_file(cfg: &FileConfig) -> Option<Value> {
        let path = Self::expand_path(&cfg.path)?;
        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Write a serde_json::Value to a config file.
    fn write_file(cfg: &FileConfig, json: &Value) -> Result<(), String> {
        let path = Self::expand_path(&cfg.path)
            .ok_or_else(|| format!("Failed to resolve path: {}", cfg.path))?;
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(json)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        Ok(())
    }

    fn is_local_endpoint(ep: &str) -> bool {
        ep.contains("127.0.0.1") || ep.contains("localhost")
    }

    /// Try to read an endpoint from a FileConfig, checking read paths in order.
    fn read_endpoint_from(cfg: &FileConfig) -> Option<String> {
        let rw = cfg.endpoint.as_ref()?;
        let json = Self::read_file(cfg)?;

        for path in &rw.read {
            if let Some(val) = json_path::json_get(&json, path) {
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }

        // Fallback to environment variable
        if let Some(ref env_key) = rw.read_env_fallback {
            if let Ok(val) = std::env::var(env_key) {
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }

        None
    }

    /// Try to read a model from a FileConfig.
    fn read_model_from(cfg: &FileConfig) -> Option<String> {
        let mc = cfg.model.as_ref()?;
        let json = Self::read_file(cfg)?;

        for path in &mc.read {
            if let Some(val) = json_path::json_get(&json, path) {
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }

        if let Some(ref env_key) = mc.read_env_fallback {
            if let Ok(val) = std::env::var(env_key) {
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }

        None
    }
}

impl ToolAdapter for DeclarativeAdapter {
    fn id(&self) -> &str { &self.manifest.id }
    fn name(&self) -> &str { &self.manifest.name }

    fn config_path(&self) -> &str {
        self.manifest.config.as_ref()
            .map(|c| c.path.as_str())
            .unwrap_or("")
    }

    fn detect(&self) -> bool {
        match &self.manifest.detect {
            DetectConfig::DirExists { path } => {
                Self::expand_path(path).map(|p| p.exists()).unwrap_or(false)
            }
            DetectConfig::FileExists { path } => {
                Self::expand_path(path).map(|p| p.exists()).unwrap_or(false)
            }
            DetectConfig::AnyFileExists { paths } => {
                paths.iter().any(|p| Self::expand_path(p).map(|p| p.exists()).unwrap_or(false))
            }
            DetectConfig::DirHasPrefix { dir, prefix } => {
                let dir_path = match Self::expand_path(dir) {
                    Some(d) => d,
                    None => return false,
                };
                if !dir_path.exists() {
                    return false;
                }
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        if name.to_string_lossy().starts_with(prefix.as_str()) {
                            return true;
                        }
                    }
                }
                false
            }
            DetectConfig::Always => true,
        }
    }

    fn current_endpoint(&self) -> Option<String> {
        // Check secondary configs first (higher priority, e.g. VS Code settings),
        // then primary config
        for cfg in &self.manifest.secondary_configs {
            if let Some(ep) = Self::read_endpoint_from(cfg) {
                return Some(ep);
            }
        }
        if let Some(ref cfg) = self.manifest.config {
            if let Some(ep) = Self::read_endpoint_from(cfg) {
                return Some(ep);
            }
        }
        None
    }

    fn current_model(&self) -> Option<String> {
        if let Some(ref cfg) = self.manifest.config {
            if let Some(m) = Self::read_model_from(cfg) {
                return Some(m);
            }
        }
        for cfg in &self.manifest.secondary_configs {
            if let Some(m) = Self::read_model_from(cfg) {
                return Some(m);
            }
        }
        None
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| Self::is_local_endpoint(&ep))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        for cfg in self.all_configs() {
            if let Some(path) = Self::expand_path(&cfg.path) {
                if path.exists() {
                    crate::backup::backup_config(&path, backup_dir)?;
                }
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        for cfg in self.all_configs() {
            let path = Self::expand_path(&cfg.path)
                .ok_or_else(|| format!("Failed to resolve path: {}", cfg.path))?;

            let mut json: Value = if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                serde_json::from_str(&content).unwrap_or(Value::Object(serde_json::Map::new()))
            } else {
                Value::Object(serde_json::Map::new())
            };

            if let Some(ref rw) = cfg.endpoint {
                for (json_path_str, write_value) in &rw.write {
                    let val = write_value.resolve(proxy_url);
                    json_path::json_set(&mut json, json_path_str, val.as_deref());
                }
            }

            Self::write_file(cfg, &json)?;
        }
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        for cfg in self.all_configs() {
            let path = Self::expand_path(&cfg.path)
                .ok_or_else(|| format!("Failed to resolve path: {}", cfg.path))?;

            match cfg.restore_mode {
                RestoreMode::File => {
                    crate::backup::restore_config(&path, backup_dir)?;
                }
                RestoreMode::RemoveKeys => {
                    // Restore primary file from backup, then remove specific keys from secondary
                    if !path.exists() {
                        continue;
                    }
                    let content = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                    let mut json: Value = serde_json::from_str(&content)
                        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
                    if let Some(ref rw) = cfg.endpoint {
                        for (json_path_str, write_value) in &rw.write {
                            if write_value.is_proxy_url() {
                                json_path::json_set(&mut json, json_path_str, None);
                            }
                        }
                    }
                    Self::write_file(cfg, &json)?;
                }
            }
        }
        Ok(())
    }
}

impl Plugin for DeclarativeAdapter {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: self.manifest.id.clone(),
            name: self.manifest.name.clone(),
            version: self.manifest.version.clone(),
            description: self.manifest.description.clone(),
            author: self.manifest.author.clone(),
            config_path_template: self.manifest.config.as_ref()
                .map(|c| c.path.clone())
                .unwrap_or_default(),
            categories: self.manifest.categories.clone(),
        }
    }
}

use serde_json::Value;
