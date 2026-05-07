use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[EGeg]\d{8}(?-u:\b)"
    ).expect("passport_cn regex");

    PatternRecognizer::new(EntityType::PassportCn, "PassportCnRecognizer", pattern, 0.6)
        .with_context_words(vec![
            "护照", "passport", "passport no", "passport number",
        ])
}
