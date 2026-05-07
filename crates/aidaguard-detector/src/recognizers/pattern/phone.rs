use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    // Matches Chinese mobile numbers (1xx-xxxxxxxxx) and common international formats
    let pattern = regex::Regex::new(
        r"(?-u:\b)(?:\+?86[\s-]?)?1[3-9]\d{9}(?-u:\b)"
    ).expect("phone regex");

    PatternRecognizer::new(EntityType::PhoneNumber, "PhoneNumberRecognizer", pattern, 0.65)
        .with_context_words(vec![
            "phone", "tel", "telephone", "mobile", "cell",
            "电话", "手机", "tel:", "phone:",
        ])
}


