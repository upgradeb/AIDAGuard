// T-DET-ANON: Anonymizer module tests — replace, mask, hash, encrypt, redact
use aidaguard_core::detector::{Match, Mode, Strategy};
use aidaguard_detector::anonymizer::replace;
use aidaguard_detector::anonymizer::mask;
use aidaguard_detector::anonymizer::hash;
use aidaguard_detector::anonymizer::encrypt;
use aidaguard_detector::anonymizer::AnonymizerOperator;

fn make_match(rule_id: &str, start: usize, end: usize, text: &str, strategy: Strategy) -> Match {
    Match {
        rule_id: rule_id.to_string(),
        start,
        end,
        text: text.to_string(),
        priority: 100,
        strategy,
        mode: Mode::Filter,
        confidence: None,
    }
}

// -- apply_replace tests --

#[test]
fn test_apply_replace_single_match() {
    let text = "My phone is 13800001111 call me";
    let matches = vec![make_match("phone_cn", 11, 22, "13800001111", Strategy::Placeholder)];
    let (result, placeholders_json) = replace::apply_replace(text, &matches);
    assert!(!result.contains("13800001111"));
    assert!(result.contains("[[PHONE_CN@"));
    assert!(!placeholders_json.is_empty());
    // Verify the placeholder can be parsed back
    let map: std::collections::HashMap<String, String> = serde_json::from_str(&placeholders_json).unwrap();
    assert_eq!(map.len(), 1);
    assert!(map.values().any(|v| v == "13800001111"));
}

#[test]
fn test_apply_replace_multiple_matches() {
    let text = "Email: a@b.com and phone: 13800001111";
    let matches = vec![
        make_match("email", 7, 14, "a@b.com", Strategy::Placeholder),
        make_match("phone_cn", 26, 37, "13800001111", Strategy::Placeholder),
    ];
    let (result, placeholders_json) = replace::apply_replace(text, &matches);
    assert!(!result.contains("a@b.com"));
    assert!(!result.contains("13800001111"));
    let map: std::collections::HashMap<String, String> = serde_json::from_str(&placeholders_json).unwrap();
    assert_eq!(map.len(), 2);
}

#[test]
fn test_apply_replace_no_matches() {
    let text = "No sensitive data here";
    let matches: Vec<Match> = vec![];
    let (result, placeholders_json) = replace::apply_replace(text, &matches);
    assert_eq!(result, text);
    assert_eq!(placeholders_json, "{}");
}

// -- apply_mask tests --

#[test]
fn test_apply_mask_phone() {
    let text = "Call 13800001111 please";
    let matches = vec![make_match("phone_cn", 5, 16, "13800001111", Strategy::Mask)];
    let (result, _) = mask::apply_mask(text, &matches);
    assert!(!result.contains("13800001111"));
    // 11 chars: keep_front=3, keep_back=3 => "138***111"
    assert!(result.contains("138***111"));
}

#[test]
fn test_apply_mask_short_value() {
    let text = "Code: abc here";
    let matches = vec![make_match("code", 6, 9, "abc", Strategy::Mask)];
    let (result, _) = mask::apply_mask(text, &matches);
    assert!(result.contains("***"));
    assert!(!result.contains("abc"));
}

#[test]
fn test_apply_mask_email() {
    let text = "Send to user@example.com now";
    let matches = vec![make_match("email", 8, 24, "user@example.com", Strategy::Mask)];
    let (result, _) = mask::apply_mask(text, &matches);
    assert!(!result.contains("user@example.com"));
    // 16 chars: keep_front=5, keep_back=5 => "user@***e.com"
    assert!(result.contains("user@***e.com"));
}

#[test]
fn test_apply_mask_preserves_surrounding() {
    let text = "prefix 13800001111 suffix";
    let matches = vec![make_match("phone_cn", 7, 18, "13800001111", Strategy::Mask)];
    let (result, _) = mask::apply_mask(text, &matches);
    assert!(result.starts_with("prefix "));
    assert!(result.ends_with(" suffix"));
}

// -- apply_hash tests --

#[test]
fn test_apply_hash_deterministic() {
    let text = "My SSN is 123-45-6789";
    let matches = vec![make_match("us_ssn", 10, 21, "123-45-6789", Strategy::Placeholder)];
    let (result1, _) = hash::apply_hash(text, &matches);
    let (result2, _) = hash::apply_hash(text, &matches);
    // Same input should produce same hash
    assert_eq!(result1, result2);
    assert!(result1.contains("<HASH:"));
}

#[test]
fn test_apply_hash_different_inputs() {
    let text_a = "Value: AAAA";
    let text_b = "Value: BBBB";
    let matches_a = vec![make_match("test", 7, 11, "AAAA", Strategy::Placeholder)];
    let matches_b = vec![make_match("test", 7, 11, "BBBB", Strategy::Placeholder)];
    let (result_a, _) = hash::apply_hash(text_a, &matches_a);
    let (result_b, _) = hash::apply_hash(text_b, &matches_b);
    assert_ne!(result_a, result_b);
}

// -- apply_encrypt tests --

#[test]
fn test_apply_encrypt_produces_enc_tag() {
    let text = "Card: 4111111111111111";
    let matches = vec![make_match("credit_card", 6, 22, "4111111111111111", Strategy::Placeholder)];
    let (result, _) = encrypt::apply_encrypt(text, &matches);
    assert!(result.contains("<ENC:"));
    assert!(!result.contains("4111111111111111"));
}

#[test]
fn test_apply_encrypt_different_each_call() {
    let text = "Card: 4111111111111111";
    let matches = vec![make_match("credit_card", 6, 22, "4111111111111111", Strategy::Placeholder)];
    let (result1, _) = encrypt::apply_encrypt(text, &matches);
    let (result2, _) = encrypt::apply_encrypt(text, &matches);
    // Due to random nonce, encrypted output should differ
    assert_ne!(result1, result2);
}

// -- AnonymizerOperator variants --

#[test]
fn test_operator_replace() {
    let text = "Phone: 13800001111";
    let matches = vec![make_match("phone", 7, 18, "13800001111", Strategy::Placeholder)];
    let (result, placeholders) = AnonymizerOperator::Replace.apply(text, &matches);
    assert!(!result.contains("13800001111"));
    assert!(!placeholders.is_empty());
}

#[test]
fn test_operator_mask() {
    let text = "Phone: 13800001111";
    let matches = vec![make_match("phone", 7, 18, "13800001111", Strategy::Mask)];
    let (result, _) = AnonymizerOperator::Mask.apply(text, &matches);
    assert!(!result.contains("13800001111"));
}

#[test]
fn test_operator_hash() {
    let text = "Phone: 13800001111";
    let matches = vec![make_match("phone", 7, 18, "13800001111", Strategy::Placeholder)];
    let (result, _) = AnonymizerOperator::Hash.apply(text, &matches);
    assert!(result.contains("<HASH:"));
}

#[test]
fn test_operator_encrypt() {
    let text = "Phone: 13800001111";
    let matches = vec![make_match("phone", 7, 18, "13800001111", Strategy::Placeholder)];
    let (result, _) = AnonymizerOperator::Encrypt.apply(text, &matches);
    assert!(result.contains("<ENC:"));
}

#[test]
fn test_operator_redact_removes_text() {
    let text = "My SSN is 123-45-6789 end";
    let matches = vec![make_match("us_ssn", 10, 21, "123-45-6789", Strategy::Placeholder)];
    let (result, _) = AnonymizerOperator::Redact.apply(text, &matches);
    assert!(!result.contains("123-45-6789"));
    assert!(result.contains("My SSN is "));
    assert!(result.contains(" end"));
}
