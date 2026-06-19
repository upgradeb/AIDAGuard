// T-PLG-13~18: Plugin adapters — backup, restore, configure
use aidaguard_plugins::{Plugin, PluginManifest, ToolAdapter};
use std::path::Path;
use std::sync::Mutex;

struct MockTool {
    config_path: String,
    endpoint: Mutex<Option<String>>,
}

impl ToolAdapter for MockTool {
    fn id(&self) -> &str { "mock_tool" }
    fn name(&self) -> &str { "Mock Tool" }
    fn config_path(&self) -> &str { &self.config_path }
    fn detect(&self) -> bool { Path::new(&self.config_path).exists() }
    fn current_endpoint(&self) -> Option<String> { self.endpoint.lock().unwrap().clone() }
    fn current_model(&self) -> Option<String> { None }
    fn backup(&self, backup_dir: &Path) -> Result<(), String> {
        aidaguard_plugins::backup::backup_config(Path::new(&self.config_path), backup_dir)
    }
    fn configure(&self, proxy_url: &str) -> Result<(), String> {
        *self.endpoint.lock().unwrap() = Some(proxy_url.to_string());
        Ok(())
    }
    fn restore(&self, backup_dir: &Path) -> Result<(), String> {
        aidaguard_plugins::backup::restore_config(Path::new(&self.config_path), backup_dir)
    }
    fn is_configured(&self) -> bool { self.endpoint.lock().unwrap().is_some() }
}

impl Plugin for MockTool {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "mock_tool".into(), name: "Mock Tool".into(),
            version: "1.0.0".into(), description: "Mock".into(),
            author: "Test".into(), config_path_template: self.config_path.clone(),
            categories: vec!["test".into()],
        }
    }
}

fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aidaguard_test_adapter_{}", uuid::Uuid::new_v4()))
}

fn mock_tool(config_path: &str) -> MockTool {
    MockTool { config_path: config_path.to_string(), endpoint: Mutex::new(None) }
}

#[test] fn test_adapter_detect_installed() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("config.json");
    std::fs::write(&config_path, r#"{"endpoint":"https://api.openai.com"}"#).unwrap();
    let tool = mock_tool(&config_path.to_string_lossy());
    assert!(tool.detect());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_adapter_detect_not_installed() {
    let tool = mock_tool("/nonexistent/path/config.json");
    assert!(!tool.detect());
}
#[test] fn test_adapter_configure() {
    let tool = mock_tool("/tmp/test_config.json");
    assert!(!tool.is_configured());
    tool.configure("http://127.0.0.1:19000").unwrap();
    assert!(tool.is_configured());
    assert_eq!(tool.current_endpoint().unwrap(), "http://127.0.0.1:19000");
}
#[test] fn test_adapter_backup_restore() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("settings.json");
    std::fs::write(&config_path, r#"{"base_url":"https://api.openai.com"}"#).unwrap();
    let backup_dir = dir.join("backups");

    let tool = mock_tool(&config_path.to_string_lossy());
    tool.backup(&backup_dir).unwrap();
    assert!(backup_dir.join("settings.json").exists());
    tool.restore(&backup_dir).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_adapter_restore_no_backup() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("no_backup.json");
    std::fs::write(&config_path, "{}").unwrap();
    let backup_dir = dir.join("empty_backups");
    let tool = mock_tool(&config_path.to_string_lossy());
    // No backup means the config was created by configure(); restore deletes it.
    tool.restore(&backup_dir).unwrap();
    assert!(!config_path.exists());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_backup_nonexistent_source() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    assert!(aidaguard_plugins::backup::backup_config(
        Path::new("/nonexistent/file.json"), &dir).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
