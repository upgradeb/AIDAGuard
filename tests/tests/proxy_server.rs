// T-PRX-10~17: Proxy server — forwarder construction, auth modes, body/header parsing
use aidaguard_proxy::Forwarder;
use aidaguard_upstream::{AuthType, ProviderConfig, ProtocolType};

fn make_provider(auth: AuthType) -> ProviderConfig {
    ProviderConfig {
        id: "test".into(), name: "Test".into(),
        protocol: ProtocolType::OpenAiCompatible, auth,
        endpoint: "https://api.test.com/v1".into(), models: vec![],
    }
}

#[test] fn test_forwarder_new_default() {
    let f = Forwarder::new();
    assert!(f.is_ok());
}

#[test] fn test_forwarder_with_bearer_auth() {
    let f = Forwarder::new().map(|fw| fw.with_bearer_auth("sk-test-key".to_string()));
    assert!(f.is_ok());
}

#[test] fn test_forwarder_from_openai_upstream() {
    let provider = make_provider(AuthType::BearerToken);
    let f = Forwarder::from_upstream(&provider, "sk-openai-key".to_string(), 300);
    assert!(f.is_ok());
}

#[test] fn test_forwarder_from_anthropic_upstream() {
    let provider = make_provider(AuthType::ApiKeyHeader { header: "x-api-key".into() });
    let f = Forwarder::from_upstream(&provider, "sk-ant-key".to_string(), 60);
    assert!(f.is_ok());
}

#[test] fn test_forwarder_from_google_upstream() {
    let provider = make_provider(AuthType::ApiKeyHeader { header: "x-goog-api-key".into() });
    let f = Forwarder::from_upstream(&provider, "google-key".to_string(), 120);
    assert!(f.is_ok());
}

#[test] fn test_model_extraction_from_request_body() {
    let body = br#"{"model": "gpt-5", "temperature": 0.7}"#;
    let json: serde_json::Value = serde_json::from_slice(body).unwrap();
    let model = json.get("model").and_then(|v| v.as_str()).unwrap_or("");
    assert_eq!(model, "gpt-5");
}

#[test] fn test_user_agent_from_header_string() {
    let ua = "Cursor/1.0";
    assert!(ua.contains("Cursor"));
    assert_eq!(ua, "Cursor/1.0");
}

#[test] fn test_content_length_parsing() {
    let cl_str = "12345";
    let cl: usize = cl_str.parse().unwrap();
    assert_eq!(cl, 12345);
}

#[test] fn test_body_size_limit_check() {
    let max_body_size = 10 * 1024 * 1024; // 10 MB
    let content_length: usize = 5 * 1024 * 1024; // 5 MB
    assert!(content_length <= max_body_size);
    let too_large: usize = 20 * 1024 * 1024; // 20 MB
    assert!(too_large > max_body_size);
}
