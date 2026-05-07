// T-UPS-05~09: ProviderRegistry — YAML loading, validation
use aidaguard_upstream::{ProviderConfig, ProviderRegistry};

fn temp_providers_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aidaguard_test_providers_{}", uuid::Uuid::new_v4()))
}

#[test] fn test_load_provider_yaml() {
    let dir = temp_providers_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = r#"
id: test_provider
name: Test Provider
protocol: open_ai_compatible
auth:
  type: bearer_token
endpoint: https://api.test.com/v1
models:
  - id: model-1
    name: Model 1
    context: 4096
    max_output: 1024
    capabilities: [chat]
"#;
    std::fs::write(dir.join("test.yaml"), yaml).unwrap();
    let mut registry = ProviderRegistry::new();
    let count = registry.load_from_dir(&dir).unwrap();
    assert_eq!(count, 1);
    let p = registry.get("test_provider").unwrap();
    assert_eq!(p.name, "Test Provider");
    assert_eq!(p.endpoint, "https://api.test.com/v1");
    assert_eq!(p.models.len(), 1);
    assert_eq!(p.models[0].id, "model-1");
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_load_all_builtin_providers() {
    // Use UpstreamManager's built-in loading
    let mut manager = aidaguard_upstream::UpstreamManager::new();
    let count = manager.load_builtins();
    assert!(count >= 7, "expected at least 7 built-in providers, got {}", count);
    assert!(manager.find_provider("openai").is_some());
    assert!(manager.find_provider("anthropic").is_some());
    assert!(manager.find_provider("deepseek").is_some());
}
#[test] fn test_provider_yaml_missing_required() {
    let dir = temp_providers_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("bad.yaml"), "name: MissingId\nendpoint: https://example.com\n").unwrap();
    let mut registry = ProviderRegistry::new();
    assert!(registry.load_from_dir(&dir).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_provider_yaml_invalid_protocol() {
    let dir = temp_providers_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let yaml = r#"
id: bad_proto
name: Bad Protocol
protocol: invalid_protocol
auth:
  type: bearer_token
endpoint: https://example.com
"#;
    std::fs::write(dir.join("bad.yaml"), yaml).unwrap();
    let mut registry = ProviderRegistry::new();
    assert!(registry.load_from_dir(&dir).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_model_capabilities_list() {
    let yaml = r#"
id: test
name: Test
protocol: open_ai_compatible
auth:
  type: bearer_token
endpoint: https://example.com
models:
  - id: m1
    name: M1
    capabilities: [chat, vision, streaming, function_calling]
"#;
    let cfg: ProviderConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg.models[0].capabilities.len(), 4);
    assert!(cfg.models[0].capabilities.contains(&"chat".to_string()));
    assert!(cfg.models[0].capabilities.contains(&"vision".to_string()));
}
