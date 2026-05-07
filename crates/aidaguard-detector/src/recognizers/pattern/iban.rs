use std::sync::Arc;

use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;
use crate::validation::iban;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[A-Z]{2}\d{2}[A-Z0-9]{11,30}(?-u:\b)"
    ).expect("iban regex");

    PatternRecognizer::new(EntityType::Iban, "IbanRecognizer", pattern, 0.4)
        .with_validator(Arc::new(|s: &str| iban::validate_iban(s)))
        .with_context_words(vec![
            "iban", "international bank", "account number", "bic",
        ])
}
