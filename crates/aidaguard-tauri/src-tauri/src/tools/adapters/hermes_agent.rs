use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".hermes").join("config.yaml"))
}

pub struct HermesAgent;

impl HermesAgent {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for HermesAgent {
    fn id(&self) -> &str { "hermes" }
    fn name(&self) -> &str { "Hermes Agent" }
    fn config_path(&self) -> &str { "~/.hermes/config.yaml" }

    fn detect(&self) -> bool {
        home_dir().map(|h| h.join(".hermes").exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        // Simple YAML key extraction for base_url
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("base_url:") || trimmed.starts_with("baseUrl:") {
                return trimmed
                    .splitn(2, ':')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
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
                return trimmed
                    .splitn(2, ':')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string());
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
                super::super::backup::backup_config(&path, backup_dir)?;
            }
        }
        Ok(())
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("Failed to determine Hermes Agent config path".to_string())?;

        if !path.exists() {
            // Create minimal config if doesn't exist
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let content = format!("base_url: \"{}\"\n", proxy_url);
            fs::write(&path, content)
                .map_err(|e| format!("Failed to create Hermes Agent config: {}", e))?;
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read Hermes Agent config: {}", e))?;

        let mut new_lines: Vec<String> = Vec::new();
        let mut found_base_url = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("base_url:") || trimmed.starts_with("baseUrl:") {
                let indent = line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
                new_lines.push(format!("{}base_url: \"{}\"", indent, proxy_url));
                found_base_url = true;
            } else {
                new_lines.push(line.to_string());
            }
        }
        if !found_base_url {
            new_lines.push(format!("base_url: \"{}\"", proxy_url));
        }

        fs::write(&path, new_lines.join("\n") + "\n")
            .map_err(|e| format!("Failed to write Hermes Agent config: {}", e))?;
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
