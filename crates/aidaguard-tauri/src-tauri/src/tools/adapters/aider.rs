use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".aider.conf.yml"))
}

pub struct Aider;

impl Aider {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Aider {
    fn id(&self) -> &str { "aider" }
    fn name(&self) -> &str { "Aider" }
    fn config_path(&self) -> &str { "~/.aider.conf.yml" }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        // Aider YAML: openai-api-base or anthropic-api-base
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("openai-api-base:") || trimmed.starts_with("anthropic-api-base:") {
                return trimmed.splitn(2, ':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("model:") {
                return trimmed.splitn(2, ':').nth(1).map(|s| s.trim().trim_matches('"').to_string());
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
        let path = config_path().ok_or("Failed to determine Aider config path".to_string())?;
        super::super::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Aider config path".to_string())?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let content = if path.exists() {
            fs::read_to_string(&path).unwrap_or_default()
        } else {
            String::new()
        };

        let mut new_lines: Vec<String> = Vec::new();
        let mut found_openai = false;
        let mut found_anthropic = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("openai-api-base:") {
                new_lines.push(format!("openai-api-base: {}", proxy_url));
                found_openai = true;
            } else if trimmed.starts_with("anthropic-api-base:") {
                new_lines.push(format!("anthropic-api-base: {}", proxy_url));
                found_anthropic = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        // If neither exists, append openai and anthropic base url
        if !found_openai && !found_anthropic {
            new_lines.push(format!("openai-api-base: {}", proxy_url));
        }

        fs::write(&path, new_lines.join("\n"))
            .map_err(|e| format!("Failed to write Aider config: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Aider config path".to_string())?;
        super::super::backup::restore_config(&path, backup_dir)
    }
}
