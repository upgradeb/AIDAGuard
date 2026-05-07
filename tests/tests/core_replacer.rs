// T-CORE-34~43: Replacer — replace, restore, mask, mixed strategies
use aidaguard_core::detector::{Match, Mode, Strategy};
use aidaguard_core::replacer;

fn make_match(start: usize, end: usize, text: &str, rule_id: &str, strategy: Strategy) -> Match {
    Match { rule_id: rule_id.into(), start, end, text: text.into(), priority: 100, strategy, mode: Mode::Filter }
}

#[test] fn test_replace_placeholder_single() {
    let hits = vec![make_match(9, 20, "13812345678", "phone_cn", Strategy::Placeholder)];
    let (result, map) = replacer::replace("my phone 13812345678 pls", &hits);
    assert!(!result.contains("13812345678"));
    assert!(result.starts_with("my phone [[PHONE_CN@"));
    assert_eq!(map.len(), 1);
}
#[test] fn test_replace_placeholder_multiple() {
    let hits = vec![
        make_match(6, 17, "13812345678", "phone_cn", Strategy::Placeholder),
        make_match(24, 40, "test@example.com", "email", Strategy::Placeholder),
    ];
    let (result, map) = replacer::replace("phone 13812345678 email test@example.com", &hits);
    assert!(!result.contains("13812345678"));
    assert!(!result.contains("test@example.com"));
    assert_eq!(map.len(), 2);
}
#[test] fn test_replace_then_restore() {
    let original = "phone 13812345678 email test@example.com";
    let hits = vec![
        make_match(6, 17, "13812345678", "phone_cn", Strategy::Placeholder),
        make_match(24, 40, "test@example.com", "email", Strategy::Placeholder),
    ];
    let (sanitized, map) = replacer::replace(original, &hits);
    let restored = replacer::restore(&sanitized, &map);
    assert_eq!(restored, original);
}
#[test] fn test_mask_phone() {
    let result = replacer::mask_value("13812345678");
    assert!(result.contains("***"));
    assert!(result.starts_with("138"));
    assert!(!result.contains("13812345678"));
}
#[test] fn test_mask_short() {
    let result = replacer::mask_value("ab");
    assert_eq!(result, "**");
}
#[test] fn test_no_matches() {
    let (result, map) = replacer::replace("no sensitive data", &[]);
    assert_eq!(result, "no sensitive data");
    assert_eq!(map.len(), 0);
}
#[test] fn test_restore_empty() {
    let map = replacer::PlaceholderMap::new();
    assert_eq!(replacer::restore("original text", &map), "original text");
}
#[test] fn test_mask_email() {
    let result = replacer::mask_value("test@example.com");
    assert!(result.contains("***"));
    assert!(!result.contains("test@example.com"));
}
#[test] fn test_replace_mixed_strategies() {
    let hits = vec![
        make_match(6, 17, "13812345678", "phone_cn", Strategy::Placeholder),
        make_match(25, 41, "test@example.com", "email", Strategy::Mask),
    ];
    let (result, map) = replacer::replace("phone=13812345678, email=test@example.com", &hits);
    assert!(!result.contains("13812345678"));
    assert!(!result.contains("test@example.com"));
    assert!(map.len() <= 2);
}
#[test] fn test_placeholder_uniqueness() {
    let hit1 = make_match(0, 11, "13812345678", "phone_cn", Strategy::Placeholder);
    let hit2 = make_match(11, 22, "13987654321", "phone_cn", Strategy::Placeholder);
    let hits = vec![hit1, hit2];
    let (result, map) = replacer::replace("1381234567813987654321", &hits);
    let placeholders: Vec<_> = map.placeholders().collect();
    assert_eq!(placeholders.len(), 2);
    assert_ne!(placeholders[0], placeholders[1]);
    assert!(!result.contains("13812345678"));
    assert!(!result.contains("13987654321"));
}
