use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)AKIA[0-9A-Z]{16}(?-u:\b)"
    ).expect("aws_access_key regex");

    PatternRecognizer::new(EntityType::AwsAccessKey, "AwsAccessKeyRecognizer", pattern, 0.65)
        .with_context_words(vec![
            "aws", "amazon", "access key", "access key id", "aws_access_key_id",
        ])
}
