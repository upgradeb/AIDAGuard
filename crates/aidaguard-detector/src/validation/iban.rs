/// Validate an IBAN using the mod-97 checksum (ISO 13616).
pub fn validate_iban(iban: &str) -> bool {
    let normalized: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
    if normalized.len() < 4 || normalized.len() > 34 {
        return false;
    }

    // Move first 4 chars to end and convert letters to digits (A→10, B→11, ...)
    let rearranged = format!("{}{}", &normalized[4..], &normalized[..4]);
    let mut digits = String::with_capacity(rearranged.len() * 2);

    for ch in rearranged.chars() {
        match ch {
            '0'..='9' => digits.push(ch),
            'A'..='Z' => {
                let val = ch as u32 - 'A' as u32 + 10;
                digits.push_str(&val.to_string());
            }
            'a'..='z' => {
                let val = ch as u32 - 'a' as u32 + 10;
                digits.push_str(&val.to_string());
            }
            _ => return false,
        }
    }

    // Mod-97 on large number (process in chunks to avoid overflow)
    mod_97(&digits) == 1
}

/// Compute mod-97 of a long digit string using chunking.
fn mod_97(digits: &str) -> u32 {
    let mut remainder = 0u32;
    for ch in digits.chars() {
        let d = ch.to_digit(10).unwrap();
        remainder = (remainder * 10 + d) % 97;
    }
    remainder
}

/// Return known IBAN lengths by country code, for pre-validation.
pub fn iban_length_for_country(code: &str) -> Option<usize> {
    match code.to_uppercase().as_str() {
        "AL" => Some(28), "AD" => Some(24), "AT" => Some(20), "AZ" => Some(28),
        "BH" => Some(22), "BE" => Some(16), "BA" => Some(20), "BR" => Some(29),
        "BG" => Some(22), "CR" => Some(22), "HR" => Some(21), "CY" => Some(28),
        "CZ" => Some(24), "DK" => Some(18), "DO" => Some(28), "EE" => Some(20),
        "FI" => Some(18), "FR" => Some(27), "GE" => Some(22), "DE" => Some(22),
        "GI" => Some(23), "GR" => Some(27), "GT" => Some(28), "HU" => Some(28),
        "IS" => Some(26), "IE" => Some(22), "IL" => Some(23), "IT" => Some(27),
        "KW" => Some(30), "LV" => Some(21), "LB" => Some(28), "LI" => Some(21),
        "LT" => Some(20), "LU" => Some(20), "MT" => Some(31), "MU" => Some(30),
        "MD" => Some(24), "MC" => Some(27), "ME" => Some(22), "NL" => Some(18),
        "NO" => Some(15), "PK" => Some(24), "PL" => Some(28), "PT" => Some(25),
        "RO" => Some(24), "SM" => Some(27), "SA" => Some(24), "RS" => Some(22),
        "SK" => Some(24), "SI" => Some(19), "ES" => Some(24), "SE" => Some(24),
        "CH" => Some(21), "TN" => Some(24), "TR" => Some(26), "AE" => Some(23),
        "GB" => Some(22), "VG" => Some(24),
        // Special administrative regions and territories
        "FO" => Some(18), "GL" => Some(18),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_iban_gb() {
        assert!(validate_iban("GB29NWBK60161331926819"));
    }

    #[test]
    fn test_valid_iban_de() {
        assert!(validate_iban("DE89370400440532013000"));
    }

    #[test]
    fn test_valid_iban_fr() {
        assert!(validate_iban("FR1420041010050500013M02606"));
    }

    #[test]
    fn test_invalid_iban() {
        assert!(!validate_iban("GB29NWBK60161331926810"));
    }

    #[test]
    fn test_iban_with_spaces() {
        assert!(validate_iban("GB29 NWBK 6016 1331 9268 19"));
    }

    #[test]
    fn test_short_iban() {
        assert!(!validate_iban("GB29"));
    }

    #[test]
    fn test_iban_length_known() {
        assert_eq!(iban_length_for_country("GB"), Some(22));
        assert_eq!(iban_length_for_country("DE"), Some(22));
    }

    #[test]
    fn test_iban_length_unknown() {
        assert_eq!(iban_length_for_country("XX"), None);
    }
}
