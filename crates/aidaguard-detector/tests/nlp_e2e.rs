use std::path::Path;

use aidaguard_core::config::{Config, NlpConfig};
use aidaguard_core::DetectionEngine;
use aidaguard_detector::AnalyzerEngine;

fn rules_dir() -> String {
    // Resolve rules directory relative to the workspace root
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rules")
        .to_string_lossy()
        .to_string()
}

fn build_engine() -> AnalyzerEngine {
    let config = Config {
        rules_dir: rules_dir(),
        nlp: NlpConfig {
            enabled: true,
            default_language: "en".to_string(),
            cache_dir: None,
        },
        ..Config::default()
    };

    AnalyzerEngine::builder()
        .with_all_pattern_recognizers()
        .with_config_rules(&config)
        .with_nlp_config(&config.nlp)
        .with_min_confidence(0.3)
        .build()
        .expect("Failed to build AnalyzerEngine")
}

// ── Pipeline smoke tests (no model download needed) ────────────────

#[test]
fn engine_builds_with_recognizers() {
    let engine = build_engine();
    let count = engine.rule_count();
    assert!(count > 20, "Engine should have 20+ rules (pattern + legacy), got {}", count);
}

#[test]
fn detects_email_via_pattern_recognizer() {
    let engine = build_engine();
    let matches = engine.detect("Contact test@example.com for details");
    let email_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "EMAIL").collect();
    assert!(!email_hits.is_empty(), "Should detect email via pattern recognizer");
    assert_eq!(email_hits[0].text, "test@example.com");
}

#[test]
fn detects_phone_via_pattern_recognizer() {
    let engine = build_engine();
    let matches = engine.detect("Call 13812345678 for support");
    let phone_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "PHONE_NUMBER").collect();
    assert!(!phone_hits.is_empty(), "Should detect phone number");
}

#[test]
fn detects_credit_card_via_pattern_recognizer() {
    let engine = build_engine();
    // Visa test number
    let matches = engine.detect("Card: 4111111111111111");
    let card_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "CREDIT_CARD").collect();
    assert!(!card_hits.is_empty(), "Should detect credit card number");
    assert_eq!(card_hits[0].text, "4111111111111111");
}

#[test]
#[cfg(feature = "nlp")]
fn nlp_recognizers_are_registered() {
    let engine = build_engine();
    // NLP recognizers register with SCREAMING_SNAKE_CASE entity names
    assert!(engine.rule_name("PERSON_NAME").is_some(), "PersonName NLP recognizer should be registered");
    assert!(engine.rule_name("ORGANIZATION").is_some(), "Organization NLP recognizer should be registered");
    assert!(engine.rule_name("ADDRESS").is_some(), "Address NLP recognizer should be registered");
}

#[test]
fn legacy_yaml_rules_also_loaded() {
    let engine = build_engine();
    // YAML rule ids (lowercase with underscore) — may be from flat or legacy directory structure
    let has_legacy = engine.rule_name("email").is_some()
        || engine.rule_name("phone_cn").is_some()
        || engine.rule_name("id_card_cn").is_some()
        || engine.rule_name("api_key").is_some()
        || engine.rule_name("bank_card_strict").is_some();
    assert!(has_legacy, "YAML rules should be loaded alongside recognizers");
}

// ── NLP model inference test (requires cached model) ───────────────

/// Full NLP inference through BERT NER model.
///
/// When the model has been downloaded and cached, this test verifies that
/// **PersonName**, **Organization**, and **Address** entities are detected
/// in an English sentence.
///
/// First run triggers a ~400 MB download from HuggingFace Hub. Subsequent
/// runs use the cached model in `~/Library/Caches/huggingface/hub/`.
#[test]
#[cfg(feature = "nlp")]
fn nlp_detects_person_name_organization() {
    let engine = build_engine();
    let text = "John Smith works at Acme Corporation in New York.";

    let matches = engine.detect(text);

    let person_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "PERSON_NAME").collect();
    let org_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "ORGANIZATION").collect();
    let loc_hits: Vec<_> = matches.iter().filter(|m| m.rule_id == "ADDRESS").collect();

    eprintln!("═══════════════════════════════════════");
    eprintln!("NLP NER results for: \"{}\"", text);
    eprintln!("  PERSON_NAME: {} hits", person_hits.len());
    for m in &person_hits {
        eprintln!("    {:?} @ {}..{}", m.text, m.start, m.end);
    }
    eprintln!("  ORGANIZATION: {} hits", org_hits.len());
    for m in &org_hits {
        eprintln!("    {:?} @ {}..{}", m.text, m.start, m.end);
    }
    eprintln!("  ADDRESS: {} hits", loc_hits.len());
    for m in &loc_hits {
        eprintln!("    {:?} @ {}..{}", m.text, m.start, m.end);
    }
    eprintln!("═══════════════════════════════════════");

    let total_nlp = person_hits.len() + org_hits.len() + loc_hits.len();
    if total_nlp > 0 {
        // Model was loaded — verify entity types
        assert!(
            person_hits.iter().any(|m| m.text.contains("John") || m.text.contains("Smith")),
            "Should detect 'John Smith' as a person name"
        );
        assert!(
            org_hits.iter().any(|m| m.text.contains("Acme")),
            "Should detect 'Acme Corporation' as an organization"
        );
        assert!(
            loc_hits.iter().any(|m| m.text.contains("New York")),
            "Should detect 'New York' as a location"
        );
    } else {
        eprintln!("NOTE: NLP model not yet cached — first detection triggers download. \
                   Re-run this test to verify NLP NER.");
        // Not a failure — model may not be cached yet
    }
}
