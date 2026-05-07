use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)(?:[13][a-km-zA-HJ-NP-Z1-9]{25,34}|0x[a-fA-F0-9]{40}|T[A-Za-z1-9]{33}|X[1-9A-HJ-NP-Za-km-z]{42})(?-u:\b)"
    ).expect("crypto_address regex");

    PatternRecognizer::new(EntityType::CryptoAddress, "CryptoAddressRecognizer", pattern, 0.55)
        .with_context_words(vec![
            "wallet", "address", "btc", "eth", "bitcoin", "ethereum",
            "crypto", "钱包", "地址", "deposit address", "withdraw",
        ])
}
