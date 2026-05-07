pub mod pattern_recognizer;

// Structured (regex + checksum)
pub mod credit_card;
pub mod id_card_cn;
pub mod passport_cn;
pub mod us_ssn;
pub mod uk_nino;
pub mod iban;
pub mod swift_code;
pub mod phone;
pub mod car_plate;
pub mod bank_account;
pub mod amount;
pub mod crypto_address;

// Network (regex only)
pub mod email;
pub mod ip_address;
pub mod mac_address;
pub mod url;
pub mod api_key;
pub mod jwt;
pub mod aws_access_key;
pub mod private_key;
