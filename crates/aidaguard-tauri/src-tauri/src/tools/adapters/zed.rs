use std::fs;
use std::path::PathBuf;
use crate::tools::home_dir;
use super::super::ToolAdapter;

fn config_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".config").join("zed").join("settings.json"))
}

pub struct Zed;

impl Zed {
    pub fn new() -> Self { Self }
}

impl ToolAdapter for Zed {
    fn id(&self) -> &str { "zed" }
    fn name(&self) -> &str { "Zed" }
    fn config_path(&self) -> &str { "~/.config/zed/settings.json" }

    fn detect(&self) -> bool {
        config_path().map(|p| p.exists()).unwrap_or(false)
    }

    fn current_endpoint(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("openai").and_then(|o| o.get("api_url"))
            .or_else(|| json.get("openai_api_url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn current_model(&self) -> Option<String> {
        let path = config_path()?;
        let content = fs::read_to_string(&path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        json.get("openai").and_then(|o| o.get("model"))
            .or_else(|| json.get("model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn is_configured(&self) -> bool {
        self.current_endpoint()
            .map(|ep| ep.contains("127.0.0.1") || ep.contains("localhost"))
            .unwrap_or(false)
    }

    fn backup(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Zed 配置路径".to_string())?;
        super::super::backup::backup_config(&path, backup_dir)
    }

    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Zed 配置路径".to_string())?;
        let content = if path.exists() {
            fs::read_to_string(&path).unwrap_or_default()
        } else {
            String::from("{}")
        };
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or(serde_json::json!({}));

        // Zed 的 openai 配置可以是嵌套对象或顶级键
        if let Some(obj) = json.as_object_mut() {
            // 顶级 openai_api_url
            obj.insert("openai_api_url".to_string(), serde_json::Value::String(proxy_url.to_string()));

            // 嵌套 openai.api_url
            if let Some(openai) = obj.get_mut("openai").and_then(|o| o.as_object_mut()) {
                openai.insert("api_url".to_string(), serde_json::Value::String(proxy_url.to_string()));
            } else {
                obj.insert("openai".to_string(), serde_json::json!({
                    "api_url": proxy_url
                }));
            }
        }

        let new_content = serde_json::to_string_pretty(&json)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&path, new_content)
            .map_err(|e| format!("写入配置失败: {}", e))?;
        Ok(())
    }

    fn restore(&self, backup_dir: &std::path::Path) -> Result<(), String> {
        let path = config_path().ok_or("无法确定 Zed 配置路径".to_string())?;
        super::super::backup::restore_config(&path, backup_dir)
    }
}
