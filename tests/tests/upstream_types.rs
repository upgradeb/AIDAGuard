// T-UPS-01~04: Upstream types — ProtocolType, AuthType, ModelInfo, ProviderConfig serde
use aidaguard_upstream::{AuthType, ModelInfo, ProtocolType, UpstreamConfig};

#[test] fn test_protocol_type_serde() {
    let p: ProtocolType = serde_json::from_str("\"open_ai_compatible\"").unwrap();
    assert_eq!(p, ProtocolType::OpenAiCompatible);
    let p: ProtocolType = serde_json::from_str("\"anthropic_compatible\"").unwrap();
    assert_eq!(p, ProtocolType::AnthropicCompatible);
    let json = serde_json::to_string(&ProtocolType::OpenAiCompatible).unwrap();
    assert!(json.contains("open_ai_compatible"));
}
#[test] fn test_auth_type_serde() {
    let a: AuthType = serde_json::from_str(r#"{"type":"bearer_token"}"#).unwrap();
    assert!(matches!(a, AuthType::BearerToken));
    let a: AuthType = serde_json::from_str(r#"{"type":"api_key_header","header":"x-api-key"}"#).unwrap();
    match a {
        AuthType::ApiKeyHeader { header } => assert_eq!(header, "x-api-key"),
        _ => panic!("expected ApiKeyHeader"),
    }
}
#[test] fn test_model_info_deserialize() {
    let yaml = r#"
id: gpt-5
name: GPT-5
context: 128000
max_output: 16384
capabilities: [chat, vision, function_calling, streaming]
"#;
    let m: ModelInfo = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(m.id, "gpt-5");
    assert_eq!(m.name, "GPT-5");
    assert_eq!(m.context, 128000);
    assert_eq!(m.max_output, 16384);
    assert_eq!(m.capabilities.len(), 4);
    assert!(m.capabilities.contains(&"streaming".to_string()));
}
#[test] fn test_upstream_config_from_provider() {
    let yaml = r#"
id: openai
name: OpenAI
protocol: open_ai_compatible
auth:
  type: bearer_token
endpoint: https://api.openai.com/v1
api_key: sk-test123
is_default: true
timeout_secs: 120
rate_limit_qps: 10
"#;
    let uc: UpstreamConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(uc.provider.id, "openai");
    assert_eq!(uc.provider.endpoint, "https://api.openai.com/v1");
    assert_eq!(uc.api_key.unwrap(), "sk-test123");
    assert!(uc.is_default);
    assert_eq!(uc.timeout_secs, 120);
    assert_eq!(uc.rate_limit_qps, 10);
}
