// T-UPS-05~09: ProviderRegistry — YAML loading, validation
use aidaguard_upstream::{AuthType, ProtocolType, ProviderConfig, ProviderRegistry};

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

// T-UPS-REG: ProviderRegistry — register, overwrite, list_ids, len, is_empty, iter, new

#[test] fn test_registry_register() {
    let mut registry = ProviderRegistry::new();
    let config = ProviderConfig {
        id: "test_reg".into(),
        name: "Test Register".into(),
        protocol: ProtocolType::OpenAiCompatible,
        auth: AuthType::BearerToken,
        endpoint: "https://api.test.com/v1".into(),
        models: vec![],
    };
    registry.register(config);
    assert!(registry.get("test_reg").is_some());
}

#[test] fn test_registry_register_overwrite() {
    let mut registry = ProviderRegistry::new();
    let config_v1 = ProviderConfig {
        id: "myprov".into(),
        name: "V1".into(),
        protocol: ProtocolType::OpenAiCompatible,
        auth: AuthType::BearerToken,
        endpoint: "https://v1.example.com".into(),
        models: vec![],
    };
    registry.register(config_v1);
    let config_v2 = ProviderConfig {
        id: "myprov".into(),
        name: "V2".into(),
        protocol: ProtocolType::OpenAiCompatible,
        auth: AuthType::BearerToken,
        endpoint: "https://v2.example.com".into(),
        models: vec![],
    };
    registry.register(config_v2);
    let p = registry.get("myprov").unwrap();
    assert_eq!(p.name, "V2");
    assert_eq!(p.endpoint, "https://v2.example.com");
}

#[test] fn test_registry_list_ids() {
    let mut registry = ProviderRegistry::new();
    registry.register(ProviderConfig {
        id: "alpha".into(), name: "Alpha".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://alpha.example.com".into(), models: vec![],
    });
    registry.register(ProviderConfig {
        id: "beta".into(), name: "Beta".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://beta.example.com".into(), models: vec![],
    });
    let ids = registry.list_ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"alpha"));
    assert!(ids.contains(&"beta"));
}

#[test] fn test_registry_len() {
    let mut registry = ProviderRegistry::new();
    assert_eq!(registry.len(), 0);
    registry.register(ProviderConfig {
        id: "one".into(), name: "One".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://one.example.com".into(), models: vec![],
    });
    assert_eq!(registry.len(), 1);
    registry.register(ProviderConfig {
        id: "two".into(), name: "Two".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://two.example.com".into(), models: vec![],
    });
    assert_eq!(registry.len(), 2);
}

#[test] fn test_registry_is_empty() {
    let registry = ProviderRegistry::new();
    assert!(registry.is_empty());
}

#[test] fn test_registry_iter() {
    let mut registry = ProviderRegistry::new();
    registry.register(ProviderConfig {
        id: "a".into(), name: "A".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://a.example.com".into(), models: vec![],
    });
    registry.register(ProviderConfig {
        id: "b".into(), name: "B".into(),
        protocol: ProtocolType::OpenAiCompatible, auth: AuthType::BearerToken,
        endpoint: "https://b.example.com".into(), models: vec![],
    });
    let count = registry.iter().count();
    assert_eq!(count, 2);
}

#[test] fn test_registry_new_empty() {
    let registry = ProviderRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    assert!(registry.list_ids().is_empty());
}
