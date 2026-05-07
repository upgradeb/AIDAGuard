// T-CORE-01~03: Match / Strategy / Mode
use aidaguard_core::detector::{Match, Mode, Strategy};

#[test]
fn test_match_creation() {
    let m = Match {
        rule_id: "phone_cn".into(), start: 9, end: 20,
        text: "13812345678".into(), priority: 100,
        strategy: Strategy::Placeholder, mode: Mode::Filter,
    };
    assert_eq!(m.rule_id, "phone_cn");
    assert_eq!(m.text, "13812345678");
}

#[test]
fn test_strategy_serde() {
    let placeholder = serde_json::to_string(&Strategy::Placeholder).unwrap();
    assert!(placeholder.contains("placeholder"));
    let mask = serde_json::to_string(&Strategy::Mask).unwrap();
    assert!(mask.contains("mask"));
    let p: Strategy = serde_json::from_str("\"placeholder\"").unwrap();
    assert_eq!(p, Strategy::Placeholder);
    let m: Strategy = serde_json::from_str("\"mask\"").unwrap();
    assert_eq!(m, Strategy::Mask);
}

#[test]
fn test_mode_serde() {
    let detect = serde_json::to_string(&Mode::Detect).unwrap();
    assert!(detect.contains("detect"));
    let filter = serde_json::to_string(&Mode::Filter).unwrap();
    assert!(filter.contains("filter"));
    let d: Mode = serde_json::from_str("\"detect\"").unwrap();
    assert_eq!(d, Mode::Detect);
    let f: Mode = serde_json::from_str("\"filter\"").unwrap();
    assert_eq!(f, Mode::Filter);
}
