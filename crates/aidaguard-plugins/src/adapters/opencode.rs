use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn config_path() -> Option<PathBuf> {
    // Check XDG_CONFIG_HOME first, fall back to ~/.config
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(&xdg).join("opencode").join("opencode.json");
        if p.exists() {
            return Some(p);
        }
    }
    home_dir().map(|h| h.join(".config").join("opencode").join("opencode.json"))
}

pub struct OpenCode;

impl OpenCode {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for OpenCode {
    fn id(&self) -> &str { "opencode" }
    fn name(&self) -> &str { "OpenCode" }
    fn config_path(&self) -> &str { "~/.config/opencode/opencode.json" }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        // OpenCode structure: provider.<name>.options.baseURL
        let providers = json.get("provider")?.as_object()?;
        for (_key, p) in providers {
            if let Some(base) = p.get("options")?.get("baseURL") {
                return base.as_str().map(|s| s.to_string());
            }
        }
        None
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        // OpenCode models are per-provider; get first model from first provider
        let providers = json.get("provider")?.as_object()?;
        for (_key, p) in providers {
            if let Some(models) = p.get("models")?.as_object() {
                return models.keys().next().cloned();
            }
        }
        None
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        if let Some(path) = config_path() {
            if path.exists() {
                crate::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine OpenCode config path".to_string())?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if !path.exists() {
            // Create minimal config
            let config = serde_json::json!({
                "provider": {
                    "aidaguard": {
                        "npm": "@ai-sdk/openai-compatible",
                        "name": "Aidaguard",
                        "options": {
                            "baseURL": proxy_url,
                            "apiKey": "aidaguard"
                        },
                        "models": {
                            "default": { "name": "default" }
                        }
                    }
                }
            });
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("Serialization failed: {}", e))?;
            fs::write(&path, content)
                .map_err(|e| format!("Failed to create OpenCode config: {}", e))?;
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read OpenCode config: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse OpenCode config: {}", e))?;

        // Update all providers' baseURL
        if let Some(providers) = json.get_mut("provider").and_then(|p| p.as_object_mut()) {
            for (_key, provider) in providers.iter_mut() {
                if let Some(options) = provider.get_mut("options").and_then(|o| o.as_object_mut()) {
                    options.insert("baseURL".to_string(), serde_json::Value::String(proxy_url.to_string()));
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
                crate::backup::restore_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }
}

impl Plugin for OpenCode {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "opencode".into(),
            name: "OpenCode".into(),
            version: "1.0.0".into(),
            description: "Open-source AI coding agent".into(),
            author: "OpenCode Community".into(),
            config_path_template: "~/.config/opencode/config.toml".into(),
            categories: vec!["cli-tool".into(), "openai-compatible".into()],
        }
    }
}
