use serde::{Deserialize, Serialize};

/// Category of sensitive data entity, determining detection strategy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityCategory {
    /// Regex + checksum validation (credit cards, IDs, phone numbers)
    Structured,
    /// NLP NER required (names, addresses, organizations, dates)
    Unstructured,
    /// Regex only (email, IP, URLs, API keys)
    Network,
}

impl std::fmt::Display for EntityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structured => write!(f, "structured"),
            Self::Unstructured => write!(f, "unstructured"),
            Self::Network => write!(f, "network"),
        }
    }
}

/// Granular entity type for sensitive data detection.
///
/// Organized by detection strategy:
/// - Structured: regex + checksum (Luhn, mod-N, etc.)
/// - Unstructured: NLP NER required
/// - Network/System: regex only
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    // ── Structured (Regex + Checksum) ──
    CreditCard,
    IdCardCn,
    PassportCn,
    UsSsn,
    UkNino,
    Iban,
    SwiftCode,
    PhoneNumber,
    CarPlate,

    // ── Unstructured (NLP NER) ──
    PersonName,
    Address,
    Organization,
    DateOfBirth,
    Age,
    Nationality,
    Religion,
    MedicalTerm,
    Occupation,
    Education,

    // ── Network / System (Regex) ──
    Email,
    IpAddress,
    MacAddress,
    Url,
    ApiKey,
    Jwt,
    AwsAccessKey,
    PrivateKey,

    // ── Financial ──
    BankAccount,
    Amount,
    CryptoAddress,

    // ── Extension ──
    Custom(String),
}

impl EntityType {
    /// Returns the category of this entity type.
    pub fn category(&self) -> EntityCategory {
        match self {
            Self::CreditCard
            | Self::IdCardCn
            | Self::PassportCn
            | Self::UsSsn
            | Self::UkNino
            | Self::Iban
            | Self::SwiftCode
            | Self::PhoneNumber
            | Self::CarPlate => EntityCategory::Structured,

            Self::PersonName
            | Self::Address
            | Self::Organization
            | Self::DateOfBirth
            | Self::Age
            | Self::Nationality
            | Self::Religion
            | Self::MedicalTerm
            | Self::Occupation
            | Self::Education => EntityCategory::Unstructured,

            Self::Email
            | Self::IpAddress
            | Self::MacAddress
            | Self::Url
            | Self::ApiKey
            | Self::Jwt
            | Self::AwsAccessKey
            | Self::PrivateKey => EntityCategory::Network,

            Self::BankAccount | Self::Amount | Self::CryptoAddress => EntityCategory::Structured,

            Self::Custom(_) => EntityCategory::Structured,
        }
    }

    /// Returns the string identifier for this entity type.
    pub fn as_str(&self) -> &str {
        match self {
            Self::CreditCard => "CREDIT_CARD",
            Self::IdCardCn => "ID_CARD_CN",
            Self::PassportCn => "PASSPORT_CN",
            Self::UsSsn => "US_SSN",
            Self::UkNino => "UK_NINO",
            Self::Iban => "IBAN",
            Self::SwiftCode => "SWIFT_CODE",
            Self::PhoneNumber => "PHONE_NUMBER",
            Self::CarPlate => "CAR_PLATE",
            Self::PersonName => "PERSON_NAME",
            Self::Address => "ADDRESS",
            Self::Organization => "ORGANIZATION",
            Self::DateOfBirth => "DATE_OF_BIRTH",
            Self::Age => "AGE",
            Self::Nationality => "NATIONALITY",
            Self::Religion => "RELIGION",
            Self::MedicalTerm => "MEDICAL_TERM",
            Self::Occupation => "OCCUPATION",
            Self::Education => "EDUCATION",
            Self::Email => "EMAIL",
            Self::IpAddress => "IP_ADDRESS",
            Self::MacAddress => "MAC_ADDRESS",
            Self::Url => "URL",
            Self::ApiKey => "API_KEY",
            Self::Jwt => "JWT",
            Self::AwsAccessKey => "AWS_ACCESS_KEY",
            Self::PrivateKey => "PRIVATE_KEY",
            Self::BankAccount => "BANK_ACCOUNT",
            Self::Amount => "AMOUNT",
            Self::CryptoAddress => "CRYPTO_ADDRESS",
            Self::Custom(_) => "CUSTOM",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(name) => write!(f, "custom:{}", name),
            other => write!(f, "{}", other.as_str()),
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CREDIT_CARD" => Ok(Self::CreditCard),
            "ID_CARD_CN" => Ok(Self::IdCardCn),
            "PASSPORT_CN" => Ok(Self::PassportCn),
            "US_SSN" => Ok(Self::UsSsn),
            "UK_NINO" => Ok(Self::UkNino),
            "IBAN" => Ok(Self::Iban),
            "SWIFT_CODE" => Ok(Self::SwiftCode),
            "PHONE_NUMBER" => Ok(Self::PhoneNumber),
            "CAR_PLATE" => Ok(Self::CarPlate),
            "PERSON_NAME" => Ok(Self::PersonName),
            "ADDRESS" => Ok(Self::Address),
            "ORGANIZATION" => Ok(Self::Organization),
            "DATE_OF_BIRTH" => Ok(Self::DateOfBirth),
            "AGE" => Ok(Self::Age),
            "NATIONALITY" => Ok(Self::Nationality),
            "RELIGION" => Ok(Self::Religion),
            "MEDICAL_TERM" => Ok(Self::MedicalTerm),
            "OCCUPATION" => Ok(Self::Occupation),
            "EDUCATION" => Ok(Self::Education),
            "EMAIL" => Ok(Self::Email),
            "IP_ADDRESS" => Ok(Self::IpAddress),
            "MAC_ADDRESS" => Ok(Self::MacAddress),
            "URL" => Ok(Self::Url),
            "API_KEY" => Ok(Self::ApiKey),
            "JWT" => Ok(Self::Jwt),
            "AWS_ACCESS_KEY" => Ok(Self::AwsAccessKey),
            "PRIVATE_KEY" => Ok(Self::PrivateKey),
            "BANK_ACCOUNT" => Ok(Self::BankAccount),
            "AMOUNT" => Ok(Self::Amount),
            "CRYPTO_ADDRESS" => Ok(Self::CryptoAddress),
            _ => Err(format!("Unknown entity type: {}", s)),
        }
    }
}
