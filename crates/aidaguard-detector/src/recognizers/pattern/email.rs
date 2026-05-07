use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?-u:\b)"
    ).expect("email regex");

    PatternRecognizer::new(EntityType::Email, "EmailRecognizer", pattern, 0.85)
        .with_context_words(vec![
            "email", "e-mail", "邮箱", "mail", "@",
        ])
}
