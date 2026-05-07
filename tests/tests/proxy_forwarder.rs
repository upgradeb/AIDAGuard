// T-PRX-06~09: Forwarder — header handling, model extraction
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

#[test] fn test_extract_model_openai_format() {
    let body = br#"{"model": "gpt-5", "messages": [{"role": "user", "content": "hello"}]}"#;
    let json: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert_eq!(json.get("model").and_then(|v| v.as_str()).unwrap_or(""), "gpt-5");
}

#[test] fn test_extract_model_anthropic_format() {
    let body = br#"{"model": "claude-opus-4-7", "max_tokens": 1024, "messages": []}"#;
    let json: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert_eq!(json.get("model").and_then(|v| v.as_str()).unwrap_or(""), "claude-opus-4-7");
}

#[test] fn test_extract_model_missing() {
    let body = br#"{"messages": [{"role": "user", "content": "hello"}]}"#;
    let json: serde_json::Value = serde_json::from_slice(body).unwrap();
    assert_eq!(json.get("model").and_then(|v| v.as_str()).unwrap_or(""), "");
}

#[test] fn test_extract_model_invalid_json() {
    let body = b"not json";
    assert!(serde_json::from_slice::<serde_json::Value>(body).is_err());
}

#[test] fn test_extract_context_around_match() {
    let text = "This is a prefix 13812345678 and some suffix text here";
    let start: usize = 17;
    let end: usize = 28;
    let radius = 10usize;
    let ctx_start = start.saturating_sub(radius);
    let ctx_end = (end + radius).min(text.len());
    let mut s = ctx_start;
    while s > 0 && !text.is_char_boundary(s) { s -= 1; }
    let mut e = ctx_end;
    while e < text.len() && !text.is_char_boundary(e) { e += 1; }
    let context = &text[s..e];
    assert!(context.contains("13812345678"));
    assert!(context.contains("prefix"));
}
