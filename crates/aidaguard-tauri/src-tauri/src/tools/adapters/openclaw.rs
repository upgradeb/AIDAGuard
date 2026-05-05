use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::{Plugin, PluginManifest, ToolAdapter};

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".openclaw").join("openclaw.json"))
}

pub struct OpenClaw;

impl OpenClaw {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for OpenClaw {
    fn id(&self) -> &str { "openclaw" }
    fn name(&self) -> &str { "OpenClaw" }
    fn config_path(&self) -> &str { "~/.openclaw/openclaw.json" }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        // Try to extract baseUrl from providers
        let providers = json.get("models")?.get("providers")?.as_object()?;
        for (_key, p) in providers {
            if let Some(base) = p.get("baseUrl").or_else(|| p.get("baseURL")) {
                return base.as_str().map(|s| s.to_string());
            }
        }
        None
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let primary = json.get("agents")?.get("defaults")?.get("model")?.get("primary")?;
        primary.as_str().map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine OpenClaw config path".to_string())?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read OpenClaw config: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse OpenClaw config: {}", e))?;

        // Update all providers' baseUrl
        if let Some(providers) = json
            .pointer_mut("/models/providers")
            .and_then(|p| p.as_object_mut())
        {
            for (_key, provider) in providers.iter_mut() {
                if let Some(obj) = provider.as_object_mut() {
                    obj.insert("baseUrl".to_string(), serde_json::Value::String(proxy_url.to_string()));
                    // Remove baseURL if exists (baseUrl takes precedence)
                    obj.remove("baseURL");
                }
            }
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("Serialization failed: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(path) = config_path() {
            if path.exists() {
                super::super::backup::restore_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }
}

impl Plugin for OpenClaw {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            version: "1.0.0".into(),
            description: "Open-source AI agent framework".into(),
            author: "OpenClaw".into(),
            config_path_template: "~/.openclaw/config.json".into(),
            categories: vec!["cli-tool".into(), "openai-compatible".into()],
        }
    }
}
