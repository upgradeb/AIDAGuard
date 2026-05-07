use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)\d{8,20}(?-u:\b)"
    ).expect("bank_account regex");

    PatternRecognizer::new(EntityType::BankAccount, "BankAccountRecognizer", pattern, 0.3)
        .with_context_words(vec![
            "bank", "account", "deposit", "银行", "账号", "账户",
            "card number", "account number", "card no", "account no",
        ])
}
