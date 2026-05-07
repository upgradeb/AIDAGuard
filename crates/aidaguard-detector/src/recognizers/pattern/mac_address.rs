use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}(?-u:\b)"
    ).expect("mac_address regex");

    PatternRecognizer::new(EntityType::MacAddress, "MacAddressRecognizer", pattern, 0.8)
        .with_context_words(vec![
            "mac", "mac address", "hardware", "ethernet", "wifi",
        ])
}
