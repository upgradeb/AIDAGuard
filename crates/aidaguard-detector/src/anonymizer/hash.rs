use aidaguard_core::detector::Match;
use sha2::{Digest, Sha256};

/// Replace matches with truncated SHA-256 hashes.
pub fn apply_hash(text: &str, matches: &[Match]) -> (String, String) {
    let mut result = text.to_string();
    let mut sorted: Vec<&Match> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));

    for m in &sorted {
        let mut hasher = Sha256::new();
        hasher.update(m.text.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let short_hash = &hash[..16.min(hash.len())];
        let replacement = format!("<HASH:{}>", short_hash);

        if m.start <= result.len() && m.end <= result.len() {
            result.replace_range(m.start..m.end, &replacement);
        }
    }

    (result, String::new())
}
