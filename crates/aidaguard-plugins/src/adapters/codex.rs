use std::fs;
use std::path::PathBuf;
use crate::home_dir;
use crate::{Plugin, PluginManifest, ToolAdapter};

fn config_path() -> Option<PathBuf> {
    let home = home_dir()?;
    let base = home.join(".codex");
    // Prefer TOML (current Codex default), then YAML, then JSON
    for ext in &["toml", "yaml", "yml", "json"] {
        let p = base.join(format!("config.{}", ext));
        if p.exists() {
            return Some(p);
        }
    }
    // Default to TOML
    Some(base.join("config.toml"))
}

pub struct Codex;

impl Codex {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Codex {
    fn id(&self) -> &str { "codex" }
    fn name(&self) -> &str { "Codex" }
    fn config_path(&self) -> &str { "~/.codex/config.toml" }

    fn detect(&self) -> bool {
        home_dir().map(|h| h.join(".codex").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(&path).ok()?;

        if path.extension().map_or(false, |e| e == "json" || e == "yaml" || e == "yml") {
            // Legacy JSON/YAML format
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            // model_providers.<id>.base_url (new format)
            if let Some(providers) = json.get("model_providers").and_then(|p| p.as_object()) {
                for (_key, p) in providers {
                    if let Some(base) = p.get("base_url").or_else(|| p.get("baseURL")) {
                        return base.as_str().map(|s| s.to_string());
                    }
                }
            }
            // providers.<id>.baseURL (old format)
            if let Some(providers) = json.get("providers").and_then(|p| p.as_object()) {
                for (_key, p) in providers {
                    if let Some(base) = p.get("baseURL").or_else(|| p.get("baseUrl")) {
                        return base.as_str().map(|s| s.to_string());
                    }
                }
            }
            return json.get("baseURL").or_else(|| json.get("baseUrl"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        // TOML format: look for base_url under [model_providers.*]
        let mut in_provider = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[model_providers.") {
                in_provider = true;
            } else if trimmed.starts_with('[') {
                in_provider = false;
            } else if in_provider && (trimmed.starts_with("base_url") || trimmed.starts_with("baseURL")) {
                return trimmed
                    .splitn(2, '=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }

        // Fallback: top-level base_url
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("base_url") || trimmed.starts_with("baseURL") {
                return trimmed
                    .splitn(2, '=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
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
                crate::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Codex config path".to_string())?;

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Strip trailing slash from proxy_url for base_url
        let base_url = proxy_url.trim_end_matches('/');

        if !path.exists() {
            // Create new TOML config with model_providers format
            let content = format!(
                r#"[model_providers.aidaguard]
name = "AIDAGuard"
base_url = "{}"
wire_api = "responses"
env_key = "AIDAGUARD_API_KEY"
"#,
                base_url
            );
            fs::write(&path, content)
                .map_err(|e| format!("Failed to create Codex config: {}", e))?;
            return Ok(());
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("toml");

        if ext == "toml" {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read Codex config: {}", e))?;

            // Check if model_providers section already exists
            let has_provider_section = content.contains("[model_providers.");

            if has_provider_section {
                // Update existing base_url values
                let mut new_lines: Vec<String> = Vec::new();
                let mut in_provider = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("[model_providers.") {
                        in_provider = true;
                        new_lines.push(line.to_string());
                    } else if trimmed.starts_with('[') {
                        in_provider = false;
                        new_lines.push(line.to_string());
                    } else if in_provider && (trimmed.starts_with("base_url") || trimmed.starts_with("baseURL")) {
                        let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                        new_lines.push(format!("{}base_url = \"{}\"", indent, base_url));
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                fs::write(&path, new_lines.join("\n") + "\n")
                    .map_err(|e| format!("Failed to write Codex config: {}", e))?;
            } else {
                // Append model_providers section
                let mut content = content;
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!(
                    r#"
[model_providers.aidaguard]
name = "AIDAGuard"
base_url = "{}"
wire_api = "responses"
env_key = "AIDAGUARD_API_KEY"
"#,
                    base_url
                ));
                fs::write(&path, content)
                    .map_err(|e| format!("Failed to write Codex config: {}", e))?;
            }
        } else {
            // JSON format
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read Codex config: {}", e))?;
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse Codex config: {}", e))?;

            // Update model_providers (new format)
            if let Some(providers) = json.get_mut("model_providers").and_then(|p| p.as_object_mut()) {
                for (_key, provider) in providers.iter_mut() {
                    if let Some(obj) = provider.as_object_mut() {
                        obj.insert("base_url".to_string(), serde_json::Value::String(base_url.to_string()));
                        obj.insert("wire_api".to_string(), serde_json::Value::String("responses".to_string()));
                    }
                }
            } else {
                // Add model_providers section
                json["model_providers"] = serde_json::json!({
                    "aidaguard": {
                        "name": "AIDAGuard",
                        "base_url": base_url,
                        "wire_api": "responses",
                        "env_key": "AIDAGUARD_API_KEY"
                    }
                });
            }

            // Also update legacy providers format if present
            if let Some(providers) = json.get_mut("providers").and_then(|p| p.as_object_mut()) {
                for (_key, provider) in providers.iter_mut() {
                    if let Some(obj) = provider.as_object_mut() {
                        obj.insert("baseURL".to_string(), serde_json::Value::String(base_url.to_string()));
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
                crate::backup::restore_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }
}

impl Plugin for Codex {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "codex".into(),
            name: "Codex".into(),
            version: "2.0.0".into(),
            description: "OpenAI Codex CLI tool (Responses API)".into(),
            author: "OpenAI".into(),
            config_path_template: "~/.codex/config.toml".into(),
            categories: vec!["cli-tool".into(), "openai-compatible".into()],
        }
    }
}
