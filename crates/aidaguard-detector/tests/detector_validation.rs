use aidaguard_detector::validation::{iban, id_card_cn, luhn};

#[test]
fn test_luhn_valid_cards() {
    // Real test card numbers (not valid for payments, but pass Luhn)
    assert!(luhn::luhn_check("4532015112830366")); // Visa
    assert!(luhn::luhn_check("5555555555554444")); // Mastercard
    assert!(luhn::luhn_check("378282246310005"));  // Amex
    assert!(luhn::luhn_check("6011111111111117")); // Discover
}

#[test]
fn test_luhn_invalid_cards() {
    assert!(!luhn::luhn_check("4532015112830367")); // Changed last digit
    assert!(!luhn::luhn_check("1234567890123456"));
    assert!(!luhn::luhn_check("1111111111111111"));
}

#[test]
fn test_luhn_edge_cases() {
    assert!(!luhn::luhn_check(""));
    assert!(!luhn::luhn_check("abc"));
    assert!(luhn::luhn_check("0"));
    // Verify checksum: sum after doubling every other from right should be divisible by 10
    assert!(luhn::luhn_check("18")); // 1*2 + 8 = 10 → valid (doubled: 2 + 8 = 10)
}

#[test]
fn test_validate_id_card_cn_valid_18() {
    // Computed valid: body 11010119900307663, remainder 10 → check '2'
    assert!(id_card_cn::validate_id_card_cn("110101199003076632"));
}

#[test]
fn test_validate_id_card_cn_invalid_check_digit() {
    assert!(!id_card_cn::validate_id_card_cn("110101199003076631"));
    assert!(!id_card_cn::validate_id_card_cn("11010119900307663X")); // Wrong check for this body
}

#[test]
fn test_validate_id_card_cn_wrong_format() {
    assert!(!id_card_cn::validate_id_card_cn("12345"));
    assert!(!id_card_cn::validate_id_card_cn("1234567890123456789")); // 19 digits
    assert!(!id_card_cn::validate_id_card_cn("abcdefghijklmnopqr")); // Non-numeric
}

#[test]
fn test_validate_id_card_cn_bad_birth_date() {
    // Month 99 doesn't exist
    assert!(!id_card_cn::validate_id_card_cn("110101199099076632"));
    // Day 32 invalid
    assert!(!id_card_cn::validate_id_card_cn("110101199003326632"));
}

#[test]
fn test_validate_iban_valid() {
    assert!(iban::validate_iban("GB29NWBK60161331926819"));
    assert!(iban::validate_iban("DE89370400440532013000"));
    assert!(iban::validate_iban("FR1420041010050500013M02606"));
    assert!(iban::validate_iban("NL91ABNA0417164300"));
    assert!(iban::validate_iban("CH9300762011623852957"));
    assert!(iban::validate_iban("IT60X0542811101000000123456"));
}

#[test]
fn test_validate_iban_case_insensitive() {
    assert!(iban::validate_iban("gb29nwbk60161331926819"));
    assert!(iban::validate_iban("de89370400440532013000"));
}

#[test]
fn test_validate_iban_invalid_mod97() {
    assert!(!iban::validate_iban("GB29NWBK60161331926810"));
    assert!(!iban::validate_iban("DE89370400440532013001"));
}

#[test]
fn test_validate_iban_too_short() {
    assert!(!iban::validate_iban("GB29"));
    assert!(!iban::validate_iban("AB"));
}
