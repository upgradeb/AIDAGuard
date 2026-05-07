use aidaguard_core::detector::Match;
use aidaguard_core::replacer;

/// Replace matches with placeholders, returning (text, placeholders_json).
pub fn apply_replace(text: &str, matches: &[Match]) -> (String, String) {
    let (result, map) = replacer::replace(text, matches);
    let placeholders = serde_json::to_string(&map.mappings()).unwrap_or_default();
    (result, placeholders)
}
