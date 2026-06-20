// T-UPS-20~22: UpstreamManager — provider resolution, endpoint matching
use aidaguard_upstream::UpstreamManager;
use aidaguard_upstream::{AuthType, ProtocolType, ProviderConfig, UpstreamConfig};

#[test] fn test_load_builtins_count() {
    let mut manager = UpstreamManager::new();
    let count = manager.load_builtins();
    assert!(count >= 7);
}
#[test] fn test_find_by_endpoint_openai() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let provider = manager.find_by_endpoint("https://api.openai.com/v1/chat/completions");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().id, "openai");
}
#[test] fn test_find_by_endpoint_anthropic() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let provider = manager.find_by_endpoint("https://api.anthropic.com/v1/messages");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().id, "anthropic");
}
#[test] fn test_find_by_endpoint_deepseek() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let provider = manager.find_by_endpoint("https://api.deepseek.com/v1/chat/completions");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().id, "deepseek");
}
#[test] fn test_find_by_endpoint_unknown() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let provider = manager.find_by_endpoint("https://unknown.example.com/v1");
    assert!(provider.is_none());
}
#[test] fn test_find_provider_by_id() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    assert!(manager.find_provider("openai").is_some());
    assert!(manager.find_provider("gemini").is_some());
    assert!(manager.find_provider("nonexistent").is_none());
}

fn make_provider_config(id: &str, endpoint: &str) -> ProviderConfig {
    ProviderConfig {
        id: id.to_string(),
        name: format!("{} provider", id),
        protocol: ProtocolType::OpenAiCompatible,
        auth: AuthType::BearerToken,
        endpoint: endpoint.to_string(),
        models: vec![],
    }
}

fn make_upstream(id: &str, endpoint: &str, api_key: &str) -> UpstreamConfig {
    UpstreamConfig {
        provider: make_provider_config(id, endpoint),
        api_key: Some(api_key.to_string()),
        is_default: false,
        timeout_secs: 60,
        rate_limit_qps: 0,
    }
}

#[test] fn test_set_upstreams() {
    let mut manager = UpstreamManager::new();
    let upstreams = vec![
        make_upstream("openai", "https://api.openai.com/v1", "sk-test1"),
        make_upstream("deepseek", "https://api.deepseek.com/v1", "sk-test2"),
    ];
    manager.set_upstreams(upstreams);
    assert_eq!(manager.upstreams().len(), 2);
}
#[test] fn test_resolve_match() {
    let mut manager = UpstreamManager::new();
    let upstreams = vec![
        make_upstream("openai", "https://api.openai.com/v1", "sk-test"),
    ];
    manager.set_upstreams(upstreams);
    let result = manager.resolve("https://api.openai.com/v1/chat/completions");
    assert!(result.is_some());
    assert_eq!(result.unwrap().provider.id, "openai");
}
#[test] fn test_resolve_no_match() {
    let mut manager = UpstreamManager::new();
    let upstreams = vec![
        make_upstream("openai", "https://api.openai.com/v1", "sk-test"),
    ];
    manager.set_upstreams(upstreams);
    let result = manager.resolve("https://unknown.example.com/v1");
    assert!(result.is_none());
}
#[test] fn test_registry_access() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let registry = manager.registry();
    assert!(registry.get("openai").is_some());
}
#[test] fn test_upstreams_empty_initially() {
    let manager = UpstreamManager::new();
    assert!(manager.upstreams().is_empty());
}
#[test] fn test_create_client_valid() {
    let mut manager = UpstreamManager::new();
    manager.load_builtins();
    let upstream = make_upstream("openai", "https://api.openai.com/v1", "sk-test-key");
    let client = manager.create_client(&upstream);
    assert!(client.is_some());
    assert_eq!(client.unwrap().provider_id(), "openai");
}
#[test] fn test_create_client_missing_provider() {
    let manager = UpstreamManager::new();
    // No builtins loaded, so no provider registry entries
    let upstream = make_upstream("nonexistent", "https://unknown.example.com/v1", "sk-test");
    let client = manager.create_client(&upstream);
    // create_client still works because it just uses the UpstreamConfig's provider
    // but the provider must be valid for the client to be created
    assert!(client.is_some());
}
