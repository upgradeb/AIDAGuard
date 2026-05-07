pub mod replace;
pub mod mask;
pub mod hash;
pub mod encrypt;

use aidaguard_core::detector::Match;

/// Operations available for anonymizing detected sensitive data.
pub enum AnonymizerOperator {
    /// Replace with a placeholder (e.g. [[EMAIL_a1b2c3d4]]).
    Replace,
    /// Partial mask (e.g. 138****5678).
    Mask,
    /// SHA-256 truncated hash.
    Hash,
    /// AES-256-GCM encryption.
    Encrypt,
    /// Complete removal (empty string).
    Redact,
}

impl AnonymizerOperator {
    /// Apply the operator to a slice of matches within the original text.
    pub fn apply(&self, text: &str, matches: &[Match]) -> (String, String) {
        match self {
            Self::Replace => replace::apply_replace(text, matches),
            Self::Mask => mask::apply_mask(text, matches),
            Self::Hash => hash::apply_hash(text, matches),
            Self::Encrypt => encrypt::apply_encrypt(text, matches),
            Self::Redact => {
                let result = apply_redact(text, matches);
                (result, String::new())
            }
        }
    }
}

fn apply_redact(text: &str, matches: &[Match]) -> String {
    let mut result = text.to_string();
    let mut sorted: Vec<&Match> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));
    for m in &sorted {
        if m.start <= result.len() && m.end <= result.len() {
            result.replace_range(m.start..m.end, "");
        }
    }
    result
}
