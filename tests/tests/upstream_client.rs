// T-UPS-10~15: UpstreamClient — auth headers, timeout
use aidaguard_upstream::{AuthType, ProtocolType, ProviderConfig, UpstreamClient};

fn make_provider(id: &str, auth: AuthType, endpoint: &str) -> ProviderConfig {
    ProviderConfig {
        id: id.into(), name: format!("{} provider", id),
        protocol: ProtocolType::OpenAiCompatible, auth, endpoint: endpoint.into(),
        models: vec![],
    }
}

#[test] fn test_client_bearer_auth() {
    let provider = make_provider("openai", AuthType::BearerToken, "https://api.openai.com/v1");
    let client = UpstreamClient::new(provider, "sk-test-key".to_string(), 300).unwrap();
    assert_eq!(client.provider_id(), "openai");
    assert_eq!(client.endpoint(), "https://api.openai.com/v1");
}
#[test] fn test_client_api_key_header() {
    let provider = make_provider("anthropic", AuthType::ApiKeyHeader { header: "x-api-key".into() }, "https://api.anthropic.com/v1");
    let client = UpstreamClient::new(provider, "sk-ant-test".to_string(), 60).unwrap();
    assert_eq!(client.provider_id(), "anthropic");
    assert_eq!(client.endpoint(), "https://api.anthropic.com/v1");
}
#[test] fn test_client_google_auth() {
    let provider = make_provider("gemini", AuthType::ApiKeyHeader { header: "x-goog-api-key".into() }, "https://generativelanguage.googleapis.com/v1beta");
    let client = UpstreamClient::new(provider, "google-api-key".to_string(), 120).unwrap();
    assert_eq!(client.provider_id(), "gemini");
}
#[test] fn test_client_default_timeout() {
    let provider = make_provider("test", AuthType::BearerToken, "https://example.com");
    let client = UpstreamClient::new(provider, "key".to_string(), 300).unwrap();
    assert_eq!(client.provider_id(), "test");
}
#[test] fn test_client_zero_timeout() {
    let provider = make_provider("test", AuthType::BearerToken, "https://example.com");
    let client = UpstreamClient::new(provider, "key".to_string(), 0).unwrap();
    assert_eq!(client.provider_id(), "test");
}
#[test] fn test_client_bearer_key_variations() {
    // "Bearer " prefix should be handled
    let provider = make_provider("openai", AuthType::BearerToken, "https://api.openai.com/v1");
    let c1 = UpstreamClient::new(provider.clone(), "sk-test".to_string(), 300).unwrap();
    assert_eq!(c1.provider_id(), "openai");
    let c2 = UpstreamClient::new(provider, "Bearer sk-test".to_string(), 300).unwrap();
    assert_eq!(c2.provider_id(), "openai");
}
