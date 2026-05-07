// T-UPS-20~22: UpstreamManager — provider resolution, endpoint matching
use aidaguard_upstream::UpstreamManager;

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
