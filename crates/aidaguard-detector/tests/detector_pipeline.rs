use aidaguard_core::detector::Match;
use aidaguard_core::DetectionEngine;
use aidaguard_detector::AnalyzerEngine;

fn build_engine() -> AnalyzerEngine {
    AnalyzerEngine::builder()
        .with_all_pattern_recognizers()
        .with_min_confidence(0.3)
        .build()
        .expect("Failed to build AnalyzerEngine")
}

#[test]
fn test_engine_builds_with_recognizers() {
    let engine = build_engine();
    assert!(engine.rule_count() >= 20, "Expected at least 20 recognizers");
}

#[test]
fn test_detect_credit_card() {
    let engine = build_engine();
    let text = "My credit card number is 4532015112830366 for visa.";
    let matches: Vec<Match> = engine.detect(text);
    let has_cc = matches.iter().any(|m| m.rule_id.contains("CREDIT_CARD"));
    assert!(has_cc, "Should detect credit card number; matches: {:?}", matches);
}

#[test]
fn test_detect_email() {
    let engine = build_engine();
    let text = "Contact us at support@example.com for help.";
    let matches: Vec<Match> = engine.detect(text);
    let has_email = matches.iter().any(|m| m.rule_id.contains("EMAIL"));
    assert!(has_email, "Should detect email; matches: {:?}", matches);
}

#[test]
fn test_detect_phone_cn() {
    let engine = build_engine();
    let text = "我的手机号是13812345678，请勿泄露。";
    let matches: Vec<Match> = engine.detect(text);
    let has_phone = matches.iter().any(|m| m.rule_id.contains("PHONE"));
    assert!(has_phone, "Should detect phone number; matches: {:?}", matches);
}

#[test]
fn test_detect_id_card_cn() {
    let engine = build_engine();
    let text = "身份证号码110101199003076632请妥善保管。";
    let matches: Vec<Match> = engine.detect(text);
    let has_id = matches.iter().any(|m| m.rule_id.contains("ID_CARD"));
    assert!(has_id, "Should detect Chinese ID card; matches: {:?}", matches);
}

#[test]
fn test_detect_iban() {
    let engine = build_engine();
    let text = "Bank transfer to IBAN GB29NWBK60161331926819 please.";
    let matches: Vec<Match> = engine.detect(text);
    let has_iban = matches.iter().any(|m| m.rule_id.contains("IBAN"));
    assert!(has_iban, "Should detect IBAN; matches: {:?}", matches);
}

#[test]
fn test_detect_jwt() {
    let engine = build_engine();
    let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
    let matches: Vec<Match> = engine.detect(text);
    let has_jwt = matches.iter().any(|m| m.rule_id.contains("JWT"));
    assert!(has_jwt, "Should detect JWT; matches: {:?}", matches);
}

#[test]
fn test_detect_ip_address() {
    let engine = build_engine();
    let text = "Server at 192.168.1.1 is unreachable.";
    let matches: Vec<Match> = engine.detect(text);
    let has_ip = matches.iter().any(|m| m.rule_id.contains("IP_ADDRESS"));
    assert!(has_ip, "Should detect IP address; matches: {:?}", matches);
}

#[test]
fn test_detect_private_key() {
    let engine = build_engine();
    let text = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA0Z3VS3mX3Lxq6Jg5Pq5q5q5q5q5q5q5q5q5q5q5q5q5q5q
-----END RSA PRIVATE KEY-----";
    let matches: Vec<Match> = engine.detect(text);
    let has_key = matches.iter().any(|m| m.rule_id.contains("PRIVATE_KEY"));
    assert!(has_key, "Should detect private key; matches: {:?}", matches);
}

#[test]
fn test_false_positive_email() {
    let engine = build_engine();
    let text = "Just some normal text without personal data.";
    let matches: Vec<Match> = engine.detect(text);
    // Email regex shouldn't match anything here
    let has_email = matches.iter().any(|m| m.rule_id.contains("EMAIL"));
    assert!(!has_email, "Should not detect email in plain text");
}

#[test]
fn test_rule_count() {
    let engine = build_engine();
    let count = engine.rule_count();
    assert!(count >= 20, "Expected 20+ recognizers in rule_count(), got {}", count);
}

#[test]
fn test_rule_name_lookup() {
    let engine = build_engine();
    let name = engine.rule_name("CREDIT_CARD");
    assert!(name.is_some(), "Should find entity name for CREDIT_CARD");
    assert_eq!(name.unwrap(), "CreditCardRecognizer");
}
