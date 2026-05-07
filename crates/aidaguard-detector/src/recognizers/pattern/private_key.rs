use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----"
    ).expect("private_key regex");

    PatternRecognizer::new(EntityType::PrivateKey, "PrivateKeyRecognizer", pattern, 0.9)
        .with_context_words(vec![
            "private key", "rsa", "ec", "dsa", "openssh",
            "pem", "-----begin", "public key",
        ])
}
