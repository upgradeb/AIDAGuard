use std::sync::Arc;

use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;
use crate::validation::id_card_cn;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[1-9]\d{5}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx](?-u:\b)"
    ).expect("id_card_cn regex");

    PatternRecognizer::new(EntityType::IdCardCn, "IdCardCnRecognizer", pattern, 0.4)
        .with_validator(Arc::new(|s: &str| id_card_cn::validate_id_card_cn(s)))
        .with_context_words(vec![
            "身份证", "公民身份号码", "id card", "identity", "identification",
        ])
}
