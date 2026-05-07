use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    // Match common API key formats: alphanumeric and common prefix/suffix patterns
    let pattern = regex::Regex::new(
        r#"(?-u:\b)(?:sk|pk|api|key|token|secret|pat)[-_][A-Za-z0-9]{16,64}(?-u:\b)"#
    ).expect("api_key regex");

    PatternRecognizer::new(EntityType::ApiKey, "ApiKeyRecognizer", pattern, 0.6)
        .with_context_words(vec![
            "api", "key", "token", "authorization", "auth", "secret",
            "api_key", "api key", "bearer",
        ])
}
