use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)\d{1,3}(?:,\d{3})*(?:\.\d{1,2})?\s*(?:USD|EUR|CNY|JPY|GBP|AUD|CAD|CHF|HKD|SGD|KRW|RUB|INR|¥|\$|€|£|元|円|₩|₹)(?-u:\b)"
    ).expect("amount regex");

    PatternRecognizer::new(EntityType::Amount, "AmountRecognizer", pattern, 0.35)
        .with_context_words(vec![
            "amount", "price", "cost", "fee", "total",
            "金额", "花费", "费用", "价格",
        ])
}
