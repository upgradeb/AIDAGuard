use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit};
use aidaguard_core::detector::Match;
use rand::RngCore;

/// Replace matches with AES-256-GCM encrypted values (Base64 encoded).
pub fn apply_encrypt(text: &str, matches: &[Match]) -> (String, String) {
    // Generate a random key per invocation — in production, this should use a key management system.
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&[0u8; 32]); // placeholder static key
    let cipher = Aes256Gcm::new(key);

    let mut result = text.to_string();
    let mut sorted: Vec<&Match> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    for m in &sorted {
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        let encrypted = cipher
            .encrypt(nonce, m.text.as_bytes())
            .unwrap_or_default();
        let encoded = base64_encode(&encrypted);
        let replacement = format!("<ENC:{}>", encoded);

        if m.start <= result.len() && m.end <= result.len() {
            result.replace_range(m.start..m.end, &replacement);
        }
    }

    (result, String::new())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
