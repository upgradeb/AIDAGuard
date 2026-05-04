use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".continue").join("config.json"))
}

pub struct ContinueDev;

impl ContinueDev {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for ContinueDev {
    fn id(&self) -> &str { "continue" }
    fn name(&self) -> &str { "Continue" }
    fn config_path(&self) -> &str { "~/.continue/config.json" }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("models")?
            .as_array()?
            .first()?
            .get("apiBase")?
            .as_str()
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("models")?
            .as_array()?
            .first()?
            .get("model")?
            .as_str()
            .map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Continue 配置路径".to_string())?;
        super::super::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Continue 配置路径".to_string())?;
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("读取 Continue 配置失败: {}", e))?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析 Continue 配置失败: {}", e))?;

        if let Some(models) = json.get_mut("models").and_then(|m| m.as_array_mut()) {
            for model in models.iter_mut() {
                if let Some(obj) = model.as_object_mut() {
                    obj.insert("apiBase".to_string(), serde_json::Value::String(proxy_url.to_string()));
                }
            }
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化 Continue 配置失败: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("写入 Continue 配置失败: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Continue 配置路径".to_string())?;
        super::super::backup::restore_config(&path, backup_dir)
    }
}
