// T-PLG-01~12: PluginRegistry — register, enable/disable, persistence
use aidaguard_plugins::{Plugin, PluginManifest, PluginRegistry};

struct MockPlugin {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    config_path_template: String,
    categories: Vec<String>,
}

impl aidaguard_plugins::ToolAdapter for MockPlugin {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn config_path(&self) -> &str { &self.config_path_template }
    fn detect(&self) -> bool { true }
    fn current_endpoint(&self) -> Option<String> { None }
    fn current_model(&self) -> Option<String> { None }
    fn backup(&self, _backup_dir: &std::path::Path) -> Result<(), String> { Ok(()) }
    fn configure(&self, _proxy_url: &str) -> Result<(), String> { Ok(()) }
    fn restore(&self, _backup_dir: &std::path::Path) -> Result<(), String> { Ok(()) }
    fn is_configured(&self) -> bool { false }
}

impl Plugin for MockPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            config_path_template: self.config_path_template.clone(),
            categories: self.categories.clone(),
        }
    }
}

fn mk_plugin(id: &str, name: &str) -> Box<dyn Plugin> {
    Box::new(MockPlugin {
        id: id.into(), name: name.into(),
        version: "1.0.0".into(), description: "A test plugin".into(),
        author: "Test".into(), config_path_template: "~/.test/config".into(),
        categories: vec!["test".into(), "cli-tool".into()],
    })
}

fn test_registry_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aidaguard_test_plugins_{}", uuid::Uuid::new_v4()))
}

#[test] fn test_register_plugin() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("test", "Test Plugin"));
    let manifests = registry.all_manifests();
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].0.id, "test");
    assert_eq!(manifests[0].0.name, "Test Plugin");
    assert!(manifests[0].1); // enabled by default
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_enable_disable_plugin() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("test", "Test"));
    assert!(registry.is_enabled("test"));
    registry.disable("test").unwrap();
    assert!(!registry.is_enabled("test"));
    registry.enable("test").unwrap();
    assert!(registry.is_enabled("test"));
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_disable_twice_ok() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("test", "Test"));
    registry.disable("test").unwrap();
    registry.disable("test").unwrap(); // should not panic
    assert!(!registry.is_enabled("test"));
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_enable_twice_ok() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("test", "Test"));
    registry.enable("test").unwrap();
    registry.enable("test").unwrap(); // should not panic
    assert!(registry.is_enabled("test"));
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_get_plugin_by_id() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("claude_code", "Claude Code"));
    assert!(registry.get("claude_code").is_some());
    assert_eq!(registry.get("claude_code").unwrap().id(), "claude_code");
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_get_nonexistent() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let registry = PluginRegistry::new(dir.clone());
    assert!(registry.get("nonexistent").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_state_persistence() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    {
        let mut registry = PluginRegistry::new(dir.clone());
        registry.register(mk_plugin("test", "Test"));
        registry.disable("test").unwrap();
    }
    // New registry from same dir should pick up disabled state
    {
        let mut registry = PluginRegistry::new(dir.clone());
        registry.register(mk_plugin("test", "Test"));
        assert!(!registry.is_enabled("test"));
    }
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_iter_enabled() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    registry.register(mk_plugin("p1", "Plugin 1"));
    registry.register(mk_plugin("p2", "Plugin 2"));
    assert_eq!(registry.iter().count(), 2);
    assert_eq!(registry.iter_enabled().count(), 2);
    registry.disable("p1").unwrap();
    assert_eq!(registry.iter().count(), 2); // iter returns all
    assert_eq!(registry.iter_enabled().count(), 1); // iter_enabled returns only enabled
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_manifest_fields() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    let p = mk_plugin("test", "Test");
    let manifest = p.manifest();
    assert_eq!(manifest.id, "test");
    assert_eq!(manifest.name, "Test");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.description, "A test plugin");
    assert_eq!(manifest.author, "Test");
    assert_eq!(manifest.config_path_template, "~/.test/config");
    assert_eq!(manifest.categories.len(), 2);
    registry.register(p);
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_disable_nonexistent() {
    let dir = test_registry_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut registry = PluginRegistry::new(dir.clone());
    assert!(registry.disable("nonexistent").is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
