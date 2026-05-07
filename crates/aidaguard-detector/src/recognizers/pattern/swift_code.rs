use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[A-Z]{4}[A-Z]{2}[A-Z0-9]{2}(?:[A-Z0-9]{3})?(?-u:\b)"
    ).expect("swift_code regex");

    PatternRecognizer::new(EntityType::SwiftCode, "SwiftCodeRecognizer", pattern, 0.7)
        .with_context_words(vec![
            "swift", "bic", "swift code", "swift/bic",
        ])
}
