use aidaguard_core::EntityType;
use aidaguard_detector::core::recognizer::Recognizer;
use aidaguard_detector::core::result::RecognizerResult;
use aidaguard_detector::recognizers::pattern;

fn get(entity: EntityType) -> Box<dyn Recognizer> {
    match entity {
        EntityType::CreditCard => Box::new(pattern::credit_card::new()),
        EntityType::Email => Box::new(pattern::email::new()),
        EntityType::PhoneNumber => Box::new(pattern::phone::new()),
        EntityType::IdCardCn => Box::new(pattern::id_card_cn::new()),
        EntityType::Iban => Box::new(pattern::iban::new()),
        EntityType::IpAddress => Box::new(pattern::ip_address::new()),
        EntityType::Url => Box::new(pattern::url::new()),
        EntityType::Jwt => Box::new(pattern::jwt::new()),
        EntityType::AwsAccessKey => Box::new(pattern::aws_access_key::new()),
        EntityType::PrivateKey => Box::new(pattern::private_key::new()),
        EntityType::UsSsn => Box::new(pattern::us_ssn::new()),
        EntityType::SwiftCode => Box::new(pattern::swift_code::new()),
        EntityType::MacAddress => Box::new(pattern::mac_address::new()),
        EntityType::CarPlate => Box::new(pattern::car_plate::new()),
        EntityType::PassportCn => Box::new(pattern::passport_cn::new()),
        EntityType::ApiKey => Box::new(pattern::api_key::new()),
        EntityType::CryptoAddress => Box::new(pattern::crypto_address::new()),
        EntityType::UkNino => Box::new(pattern::uk_nino::new()),
        EntityType::BankAccount => Box::new(pattern::bank_account::new()),
        EntityType::Amount => Box::new(pattern::amount::new()),
        _ => panic!("No pattern recognizer for {:?}", entity),
    }
}

fn assert_detects(entity: EntityType, text: &str, expected_count: usize) -> Vec<RecognizerResult> {
    let entity_str = entity.as_str().to_string();
    let recognizer = get(entity);
    let results = recognizer.analyze(text);
    assert_eq!(
        results.len(),
        expected_count,
        "{}: expected {} match(es) in '{}', got {}: {:?}",
        entity_str,
        expected_count,
        text,
        results.len(),
        results,
    );
    results
}

// ── Phone ──

#[test]
fn phone_chinese_context() {
    assert_detects(EntityType::PhoneNumber, "我的手机号是13812345678，请勿泄露。", 1);
}

#[test]
fn phone_ascii_context() {
    assert_detects(EntityType::PhoneNumber, "call 13812345678 now", 1);
}

#[test]
fn phone_international() {
    assert_detects(EntityType::PhoneNumber, "call +86 13812345678 now", 1);
}

// ── Email ──

#[test]
fn email_simple() {
    assert_detects(EntityType::Email, "contact test@example.com for info", 1);
}

#[test]
fn email_no_match_in_plain_text() {
    assert_detects(EntityType::Email, "just some normal text without personal data", 0);
}

// ── Credit Card ──

#[test]
fn credit_card_valid_visa() {
    assert_detects(EntityType::CreditCard, "card 4532015112830366 visa", 1);
}

#[test]
fn credit_card_invalid_luhn_skipped() {
    // Does not pass Luhn checksum
    assert_detects(EntityType::CreditCard, "card 1234567890123456 invalid", 0);
}

// ── ID Card CN ──

#[test]
fn id_card_cn_valid() {
    assert_detects(EntityType::IdCardCn, "身份证号码110101199003076632请妥善保管。", 1);
}

#[test]
fn id_card_cn_invalid_check_digit() {
    assert_detects(EntityType::IdCardCn, "身份证号码110101199003076631无效。", 0);
}

// ── IBAN ──

#[test]
fn iban_valid() {
    assert_detects(EntityType::Iban, "Bank transfer to IBAN GB29NWBK60161331926819 please.", 1);
}

#[test]
fn iban_invalid_mod97() {
    assert_detects(EntityType::Iban, "IBAN GB29NWBK60161331926810 invalid", 0);
}

// ── IP Address ──

#[test]
fn ipv4_valid() {
    assert_detects(EntityType::IpAddress, "Server at 192.168.1.1 is unreachable.", 1);
}

#[test]
fn ipv4_invalid_octet() {
    assert_detects(EntityType::IpAddress, "Invalid IP 999.999.999.999", 0);
}

// ── URL ──

#[test]
fn url_https() {
    assert_detects(EntityType::Url, "visit https://example.com/path for details", 1);
}

// ── JWT ──

#[test]
fn jwt_bearer_token() {
    assert_detects(EntityType::Jwt, "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U", 1);
}

// ── AWS Access Key ──

#[test]
fn aws_access_key() {
    assert_detects(EntityType::AwsAccessKey, "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE", 1);
}

// ── Private Key ──

#[test]
fn private_key_pem() {
    let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA0Z3VS3mX3Lxq6Jg5Pq5q\n-----END RSA PRIVATE KEY-----";
    assert_detects(EntityType::PrivateKey, text, 1);
}

// ── US SSN ──

#[test]
fn us_ssn_valid() {
    assert_detects(EntityType::UsSsn, "SSN: 123-45-6789 for identification", 1);
}

#[test]
fn us_ssn_invalid_area_000() {
    assert_detects(EntityType::UsSsn, "SSN: 000-45-6789 invalid", 0);
}

#[test]
fn us_ssn_invalid_area_666() {
    assert_detects(EntityType::UsSsn, "SSN: 666-45-6789 invalid", 0);
}

// ── SWIFT Code ──

#[test]
fn swift_code_valid() {
    assert_detects(EntityType::SwiftCode, "SWIFT: DEUTDEFFXXX for transfer", 1);
}

// ── MAC Address ──

#[test]
fn mac_address_colon() {
    assert_detects(EntityType::MacAddress, "MAC: 00:1A:2B:3C:4D:5E", 1);
}

#[test]
fn mac_address_hyphen() {
    assert_detects(EntityType::MacAddress, "MAC: 00-1A-2B-3C-4D-5E", 1);
}

// ── Context word boosting ──

#[test]
fn context_words_boost_confidence() {
    let recognizer = pattern::credit_card::new();
    let results = recognizer.analyze("my credit card 4532015112830366 visa");
    assert_eq!(results.len(), 1);
    // Base confidence is 0.4, context words "credit", "card", "visa" should boost above 0.4
    assert!(results[0].score > 0.4, "Context words should boost confidence above base 0.4, got {}", results[0].score);
}
