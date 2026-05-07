use std::sync::Arc;

use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

fn validate_ip(v: &str) -> bool {
    // Validates IPv4 octet ranges and basic IPv6 structure
    if let Some(caps) = regex::Regex::new(r"^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$")
        .unwrap()
        .captures(v)
    {
        for i in 1..=4 {
            let octet: u32 = caps.get(i).unwrap().as_str().parse().unwrap_or(256);
            if octet > 255 {
                return false;
            }
        }
        return true;
    }
    // For IPv6, accept the match if it contains colons and valid hex
    v.contains(':') && v.chars().all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
}

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)(?:\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}|(?:[0-9a-fA-F]{1,4}:){1,7}[0-9a-fA-F]{1,4}|(?:[0-9a-fA-F]{1,4}:){1,6}:\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})(?-u:\b)"
    ).expect("ip_address regex");

    PatternRecognizer::new(EntityType::IpAddress, "IpAddressRecognizer", pattern, 0.7)
        .with_validator(Arc::new(validate_ip))
        .with_context_words(vec![
            "ip", "address", "server", "host", "client ip",
        ])
}
