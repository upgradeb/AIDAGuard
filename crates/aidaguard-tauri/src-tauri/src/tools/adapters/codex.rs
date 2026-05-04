use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    // Codex supports .json, .yaml, .toml — check in order
    let home = home_dir()?;
    let base = home.join(".codex");
    for ext in &["json", "yaml", "toml"] {
        let p = base.join(format!("config.{}", ext));
        if p.exists() {
            return Some(p);
        }
    }
    // Default to JSON
    Some(base.join("config.json"))
}

pub struct Codex;

impl Codex {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Codex {
    fn id(&self) -> &str { "codex" }
    fn name(&self) -> &str { "Codex" }
    fn config_path(&self) -> &str { "~/.codex/config.json" }

    fn detect(&self) -> bool {
        home_dir().map(|h| h.join(".codex").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        if !path.exists() {
            return None;
        }
        // For JSON/YAML config
        if path.extension().map_or(false, |e| e == "json" || e == "yaml" || e == "yml") {
            let content = fs::read_to_string(&path).ok()?;
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            // Try providers.<default-provider>.baseURL or providers.*.baseURL
            if let Some(providers) = json.get("providers").and_then(|p| p.as_object()) {
                for (_key, p) in providers {
                    if let Some(base) = p.get("baseURL").or_else(|| p.get("baseUrl")) {
                        return base.as_str().map(|s| s.to_string());
                    }
                }
            }
            // Try top-level baseURL
            return json.get("baseURL").or_else(|| json.get("baseUrl"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        } else if path.extension().map_or(false, |e| e == "toml") {
            // Simple TOML extraction
            let content = fs::read_to_string(&path).ok()?;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("base_url") || trimmed.starts_with("baseUrl") {
                    return trimmed
                        .splitn(2, '=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
        None
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;
        if path.extension().map_or(false, |e| e == "json" || e == "yaml" || e == "yml") {
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            json.get("model").and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("model") && !trimmed.starts_with("model_provider") {
                    return trimmed
                        .splitn(2, '=')
                        .nth(1)
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
            None
        }
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
        let path = config_path().ok_or("Failed to determine Codex config path".to_string())?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if !path.exists() {
            // Create minimal config
            let config = serde_json::json!({
                "providers": {
                    "custom": {
                        "name": "Aidaguard",
                        "baseURL": proxy_url,
                        "envKey": "CUSTOM_API_KEY"
                    }
                }
            });
            let content = serde_json::to_string_pretty(&config)
                .map_err(|e| format!("Serialization failed: {}", e))?;
            fs::write(&path, content)
                .map_err(|e| format!("Failed to create Codex config: {}", e))?;
            return Ok(());
        }

        // Determine format and update
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("json");
        if ext == "toml" {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read Codex config: {}", e))?;
            let mut new_lines: Vec<String> = Vec::new();
            let mut in_provider = false;
            for line in content.lines() {
                if line.trim().starts_with("[model_providers.") {
                    in_provider = true;
                    new_lines.push(line.to_string());
                } else if in_provider && line.trim().starts_with("base_url") {
                    let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                    new_lines.push(format!("{}base_url = \"{}\"", indent, proxy_url));
                } else {
                    new_lines.push(line.to_string());
                }
            }
            fs::write(&path, new_lines.join("\n") + "\n")
                .map_err(|e| format!("Failed to write Codex config: {}", e))?;
        } else {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read Codex config: {}", e))?;
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse Codex config: {}", e))?;

            if let Some(providers) = json.get_mut("providers").and_then(|p| p.as_object_mut()) {
                for (_key, provider) in providers.iter_mut() {
                    if let Some(obj) = provider.as_object_mut() {
                        obj.insert("baseURL".to_string(), serde_json::Value::String(proxy_url.to_string()));
                    }
                }
            }

            let new_content = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Serialization failed: {}", e))?;
            fs::write(&path, new_content)
                .map_err(|e| format!("Failed to write config: {}", e))?;
        }
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
