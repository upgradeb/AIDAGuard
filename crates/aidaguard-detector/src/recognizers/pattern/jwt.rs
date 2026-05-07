use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+(?-u:\b)"
    ).expect("jwt regex");

    PatternRecognizer::new(EntityType::Jwt, "JwtRecognizer", pattern, 0.7)
        .with_context_words(vec![
            "jwt", "token", "bearer", "authorization", "auth",
        ])
}
