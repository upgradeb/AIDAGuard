use std::sync::Arc;

use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;
use crate::validation::luhn;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?x)(?-u:\b)(?:
            4[0-9]{12}(?:[0-9]{3})? |          # Visa
            5[1-5][0-9]{14} |                   # Mastercard
            3[47][0-9]{13} |                     # Amex
            6(?:011|5[0-9]{2})[0-9]{12} |       # Discover
            (?:2131|1800|35\d{3})\d{11}          # JCB
        )(?-u:\b)"
    ).expect("credit card regex");

    PatternRecognizer::new(EntityType::CreditCard, "CreditCardRecognizer", pattern, 0.4)
        .with_validator(Arc::new(|s: &str| luhn::luhn_check(s)))
        .with_context_words(vec![
            "credit", "card", "visa", "mastercard", "master", "amex",
            "american express", "discover", "jcb", "diners",
            "cvv", "expiry", "expiration",
        ])
}
