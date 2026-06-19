// T-CORE-21~33: Detector — detection, dedup, overlap, reload, modes
use aidaguard_core::detector::{Detector, Mode, RuleDef, Strategy};

fn make_rule(id: &str, name: &str, pattern: &str, strategy: Strategy, priority: u32, exclude: Option<&str>, enabled: bool, mode: Mode) -> RuleDef {
    RuleDef {
        id: id.into(), name: name.into(),
        pattern: pattern.into(), exclude: exclude.map(|s| s.into()),
        enabled, strategy, mode, priority, compliance: vec![],
        validator: None, context_words: vec![], base_confidence: None,
        region: None, source: "system".into(),
    }
}

fn make_detector() -> Detector {
    let mut d = Detector::new();
    d.add_rule(make_rule("phone", "phone", r"1[3-9]\d{9}", Strategy::Placeholder, 100, None, true, Mode::Filter)).unwrap();
    d.add_rule(make_rule("id_card", "id_card", r"\d{17}[\dXx]", Strategy::Placeholder, 100, None, true, Mode::Filter)).unwrap();
    d.add_rule(make_rule("email", "email", r"[\w.+-]+@[\w-]+\.\w+", Strategy::Mask, 90, None, true, Mode::Filter)).unwrap();
    d
}

#[test] fn test_detect_phone() {
    let d = make_detector();
    let hits = d.detect("my phone 13812345678");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rule_id, "phone");
    assert_eq!(hits[0].text, "13812345678");
}
#[test] fn test_detect_multiple() {
    let d = make_detector();
    let hits = d.detect("phone 13812345678 email test@example.com");
    assert_eq!(hits.len(), 2);
}
#[test] fn test_no_match() {
    let d = make_detector();
    let hits = d.detect("hello world");
    assert_eq!(hits.len(), 0);
}
#[test] fn test_overlap_same_priority() {
    let d = make_detector();
    let hits = d.detect("id 320102199001011234 here");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rule_id, "id_card");
}
#[test] fn test_id_card_with_x() {
    let d = make_detector();
    let hits = d.detect("number 32010219900101123X");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "32010219900101123X");
}
#[test] fn test_deduplication() {
    let d = make_detector();
    let hits = d.detect("13812345678 and 320102199001011234");
    assert_eq!(hits.len(), 2);
}
#[test] fn test_empty_input() {
    let d = make_detector();
    let hits = d.detect("");
    assert_eq!(hits.len(), 0);
}
#[test] fn test_email_exclude_retina() {
    let mut d = Detector::new();
    d.add_rule(make_rule("email", "email", r"[\w.+-]+@[\w-]+\.\w+", Strategy::Mask, 90, Some(r"@\d+x\.(?:png|jpg|jpeg|gif|svg|webp|ico|pdf)\b"), true, Mode::Filter)).unwrap();
    let hits = d.detect("contact test@example.com or 123456@qq.com");
    assert_eq!(hits.len(), 2);
    let hits = d.detect("icon file icon@2x.png and logo@3x.jpg");
    assert_eq!(hits.len(), 0);
    let hits = d.detect("photo icon@2x.png email admin@foo.com");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "admin@foo.com");
}
#[test] fn test_detect_only_mode() {
    let mut d = Detector::new();
    d.add_rule(make_rule("phone", "phone", r"1[3-9]\d{9}", Strategy::Placeholder, 100, None, true, Mode::Detect)).unwrap();
    let hits = d.detect("my phone 13812345678");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].mode, Mode::Detect);
}
#[test] fn test_priority_ordering() {
    let mut d = Detector::new();
    d.add_rule(make_rule("low", "Low", r"\d+", Strategy::Placeholder, 10, None, true, Mode::Filter)).unwrap();
    d.add_rule(make_rule("high", "High", r"\d{11}", Strategy::Placeholder, 200, None, true, Mode::Filter)).unwrap();
    let hits = d.detect("13812345678");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rule_id, "high");
}
#[test] fn test_reload_from_dir() {
    let dir = std::env::temp_dir().join("aidaguard_test_rules_reload");
    let _ = std::fs::create_dir_all(&dir);
    let yaml = r#"
version: "1"
name: reload_test
description: ""
rules:
  - id: reload_rule
    name: Reload Rule
    pattern: "TEST"
"#;
    let path = dir.join("reload_test.yaml");
    std::fs::write(&path, yaml).unwrap();
    let mut d = Detector::new();
    let count = d.load_from_dir(&dir).unwrap();
    assert!(count > 0);
    assert_eq!(d.rule_name("reload_rule"), Some("Reload Rule"));
    let _ = std::fs::remove_file(&path);
    let mut d2 = Detector::new();
    d2.add_rule(make_rule("old", "Old", "x", Strategy::Placeholder, 100, None, true, Mode::Filter)).unwrap();
    assert_eq!(d2.rule_count(), 1);
    d2.load_from_dir(&dir).unwrap();
    assert_eq!(d2.rule_count(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_reload_empty_dir() {
    let dir = std::env::temp_dir().join("aidaguard_test_empty_rules");
    let _ = std::fs::create_dir_all(&dir);
    let mut d = Detector::new();
    let count = d.load_from_dir(&dir).unwrap();
    assert_eq!(count, 0);
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_add_rule_disabled() {
    let mut d = Detector::new();
    d.add_rule(make_rule("disabled_rule", "Disabled", r"\d{11}", Strategy::Placeholder, 100, None, false, Mode::Filter)).unwrap();
    assert_eq!(d.rule_count(), 0);
    let hits = d.detect("13812345678");
    assert!(hits.is_empty());
}
