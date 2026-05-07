use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[A-Za-z]{2}\d{6}[ABCDabcd](?-u:\b)"
    ).expect("uk_nino regex");

    PatternRecognizer::new(EntityType::UkNino, "UkNinoRecognizer", pattern, 0.65)
        .with_context_words(vec![
            "national insurance", "nino", "ni number", "insurance number",
        ])
}
