use aidaguard_core::detector::Match;

/// Apply mask to each match in the text.
pub fn apply_mask(text: &str, matches: &[Match]) -> (String, String) {
    let mut result = text.to_string();
    let mut sorted: Vec<&Match> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));

    for m in &sorted {
        let masked = mask_chars(&m.text);
        if m.start <= result.len() && m.end <= result.len() {
            result.replace_range(m.start..m.end, &masked);
        }
    }

    (result, String::new())
}

fn mask_chars(text: &str) -> String {
    let len = text.chars().count();
    if len <= 3 {
        return "*".repeat(len);
    }
    let chars: Vec<char> = text.chars().collect();
    let keep_front = (len / 3).max(1);
    let keep_back = (len / 3).max(1);
    let front: String = chars[..keep_front].iter().collect();
    let back: String = chars[len - keep_back..].iter().collect();
    format!("{}***{}", front, back)
}
