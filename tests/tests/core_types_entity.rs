// T-CORE-09~14: EntityType / EntityCategory
use aidaguard_core::{EntityCategory, EntityType};

#[test] fn test_entity_type_display() {
    assert_eq!(EntityType::CreditCard.to_string(), "CREDIT_CARD");
    assert_eq!(EntityType::PersonName.to_string(), "PERSON_NAME");
    assert_eq!(EntityType::Email.to_string(), "EMAIL");
    assert_eq!(EntityType::Custom("my_type".into()).to_string(), "custom:my_type");
}
#[test] fn test_entity_category_assignment() {
    assert_eq!(EntityType::CreditCard.category(), EntityCategory::Structured);
    assert_eq!(EntityType::PersonName.category(), EntityCategory::Unstructured);
    assert_eq!(EntityType::Email.category(), EntityCategory::Network);
    assert_eq!(EntityType::ApiKey.category(), EntityCategory::Network);
}
#[test] fn test_entity_type_from_str() {
    assert_eq!("CREDIT_CARD".parse::<EntityType>().unwrap(), EntityType::CreditCard);
    assert_eq!("PERSON_NAME".parse::<EntityType>().unwrap(), EntityType::PersonName);
    assert!("INVALID_TYPE".parse::<EntityType>().is_err());
}
#[test] fn test_entity_type_serde_roundtrip() {
    let cases = vec![EntityType::CreditCard, EntityType::PersonName, EntityType::Email, EntityType::ApiKey, EntityType::Custom("test".into())];
    for original in cases {
        let json = serde_json::to_string(&original).unwrap();
        let restored: EntityType = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}
#[test] fn test_entity_type_all_variants_have_category() {
    let all = vec![
        EntityType::CreditCard, EntityType::IdCardCn, EntityType::PassportCn,
        EntityType::UsSsn, EntityType::UkNino, EntityType::Iban, EntityType::SwiftCode,
        EntityType::PhoneNumber, EntityType::CarPlate, EntityType::PersonName, EntityType::Address,
        EntityType::Organization, EntityType::DateOfBirth, EntityType::Age,
        EntityType::Nationality, EntityType::Religion, EntityType::MedicalTerm,
        EntityType::Occupation, EntityType::Education, EntityType::Email, EntityType::IpAddress,
        EntityType::MacAddress, EntityType::Url, EntityType::ApiKey, EntityType::Jwt,
        EntityType::AwsAccessKey, EntityType::PrivateKey, EntityType::BankAccount,
        EntityType::Amount, EntityType::CryptoAddress,
    ];
    for e in &all { let _cat = e.category(); let s = e.to_string(); assert!(!s.is_empty()); }
}
#[test] fn test_custom_entity_type() {
    let custom = EntityType::Custom("my_entity".into());
    assert_eq!(custom.to_string(), "custom:my_entity");
    assert_eq!(custom.as_str(), "CUSTOM");
    let json = serde_json::to_string(&custom).unwrap();
    let restored: EntityType = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, EntityType::Custom("my_entity".into()));
}
#[test] fn test_entity_type_from_str_invalid() {
    assert!("INVALID_TYPE".parse::<EntityType>().is_err());
}
#[test] fn test_entity_category_display() {
    assert_eq!(EntityCategory::Structured.to_string(), "structured");
    assert_eq!(EntityCategory::Unstructured.to_string(), "unstructured");
    assert_eq!(EntityCategory::Network.to_string(), "network");
}
#[test] fn test_entity_type_as_str_all() {
    assert_eq!(EntityType::CreditCard.as_str(), "CREDIT_CARD");
    assert_eq!(EntityType::AwsAccessKey.as_str(), "AWS_ACCESS_KEY");
    assert_eq!(EntityType::Jwt.as_str(), "JWT");
}
