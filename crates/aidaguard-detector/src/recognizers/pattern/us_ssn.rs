use std::sync::Arc;

use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

fn validate_ssn(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let area: u32 = parts[0].parse().unwrap_or(0);
    let group: u32 = parts[1].parse().unwrap_or(0);
    let serial: u32 = parts[2].parse().unwrap_or(0);

    area != 0
        && area != 666
        && area < 900
        && group != 0
        && serial != 0
}

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)\d{3}-\d{2}-\d{4}(?-u:\b)"
    ).expect("us_ssn regex");

    PatternRecognizer::new(EntityType::UsSsn, "UsSsnRecognizer", pattern, 0.5)
        .with_validator(Arc::new(validate_ssn))
        .with_context_words(vec![
            "ssn", "social security", "social security number", "tin",
        ])
}
