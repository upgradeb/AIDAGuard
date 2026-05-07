use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r#"(?-u:\b)https?://(?:www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}(?-u:\b)(?:[-a-zA-Z0-9()@:%_+.~#?&/=]*)"#
    ).expect("url regex");

    PatternRecognizer::new(EntityType::Url, "UrlRecognizer", pattern, 0.75)
        .with_context_words(vec![
            "url", "link", "website", "http", "https", "网址", "site",
        ])
}
