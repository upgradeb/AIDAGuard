// T-DET-RES: RecognizerResult — field mapping, legacy conversion, serde round-trip
use aidaguard_core::detector::{Match, Mode, Strategy};
use aidaguard_core::EntityType;
use aidaguard_detector::core::result::RecognizerResult;

#[test]
fn test_to_legacy_match_field_mapping() {
    let result = RecognizerResult {
        entity_type: EntityType::Email,
        start: 10,
        end: 24,
        text: "user@example.com".to_string(),
        score: 0.85,
        recognizer_name: "EmailRecognizer".to_string(),
    };
    let m: Match = result.to_legacy_match(Strategy::Placeholder, Mode::Filter);
    assert_eq!(m.rule_id, "EMAIL");
    assert_eq!(m.start, 10);
    assert_eq!(m.end, 24);
    assert_eq!(m.text, "user@example.com");
    assert_eq!(m.priority, 100);
    assert_eq!(m.strategy, Strategy::Placeholder);
    assert_eq!(m.mode, Mode::Filter);
    assert_eq!(m.confidence, Some(0.85));
}

#[test]
fn test_to_legacy_match_with_mask_strategy() {
    let result = RecognizerResult {
        entity_type: EntityType::PhoneNumber,
        start: 0,
        end: 11,
        text: "13800001111".to_string(),
        score: 0.9,
        recognizer_name: "PhoneCnRecognizer".to_string(),
    };
    let m: Match = result.to_legacy_match(Strategy::Mask, Mode::Filter);
    assert_eq!(m.strategy, Strategy::Mask);
    assert_eq!(m.rule_id, "PHONE_NUMBER");
}

#[test]
fn test_to_legacy_match_custom_entity_type() {
    let result = RecognizerResult {
        entity_type: EntityType::Custom("my_entity".to_string()),
        start: 0,
        end: 5,
        text: "hello".to_string(),
        score: 0.5,
        recognizer_name: "CustomRecognizer".to_string(),
    };
    let m: Match = result.to_legacy_match(Strategy::Placeholder, Mode::Detect);
    assert_eq!(m.rule_id, "custom:my_entity");
    assert_eq!(m.mode, Mode::Detect);
}

#[test]
fn test_from_regex_match_field_population() {
    let result = RecognizerResult::from_regex_match(
        EntityType::CreditCard,
        5,
        21,
        "4111111111111111",
        0.75,
        "CreditCardRecognizer",
    );
    assert_eq!(result.entity_type, EntityType::CreditCard);
    assert_eq!(result.start, 5);
    assert_eq!(result.end, 21);
    assert_eq!(result.text, "4111111111111111");
    assert!((result.score - 0.75).abs() < f64::EPSILON);
    assert_eq!(result.recognizer_name, "CreditCardRecognizer");
}

#[test]
fn test_round_trip_to_legacy_and_back() {
    let original = RecognizerResult {
        entity_type: EntityType::Iban,
        start: 3,
        end: 20,
        text: "GB29NWBK60161331926819".to_string(),
        score: 0.95,
        recognizer_name: "IbanRecognizer".to_string(),
    };
    let legacy = original.to_legacy_match(Strategy::Placeholder, Mode::Filter);
    // Verify the confidence was carried through
    assert_eq!(legacy.confidence, Some(0.95));
    assert_eq!(legacy.start, 3);
    assert_eq!(legacy.end, 20);
    assert_eq!(legacy.text, "GB29NWBK60161331926819");
}

#[test]
fn test_serde_round_trip() {
    let result = RecognizerResult {
        entity_type: EntityType::UsSsn,
        start: 0,
        end: 11,
        text: "123-45-6789".to_string(),
        score: 0.8,
        recognizer_name: "SsnRecognizer".to_string(),
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: RecognizerResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.entity_type, EntityType::UsSsn);
    assert_eq!(deserialized.start, 0);
    assert_eq!(deserialized.end, 11);
    assert_eq!(deserialized.text, "123-45-6789");
    assert!((deserialized.score - 0.8).abs() < f64::EPSILON);
    assert_eq!(deserialized.recognizer_name, "SsnRecognizer");
}
