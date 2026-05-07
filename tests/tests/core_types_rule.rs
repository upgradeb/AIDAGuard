// T-CORE-04~08: RuleDef / RuleFile / compile_regex
use aidaguard_core::detector::{self, Mode, RuleDef, RuleFile, Strategy};

#[test]
fn test_rule_def_deserialize_minimal() {
    let yaml = r#"
id: test_rule
name: Test Rule
pattern: "\\d{11}"
"#;
    let def: RuleDef = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(def.id, "test_rule");
    assert!(def.enabled);
    assert_eq!(def.priority, 100);
    assert_eq!(def.strategy, Strategy::Placeholder);
    assert_eq!(def.mode, Mode::Filter);
    assert!(def.exclude.is_none());
}

#[test]
fn test_rule_def_deserialize_full() {
    let yaml = r#"
id: email
name: Email
pattern: "[\\w.+-]+@[\\w-]+\\.\\w+"
exclude: "@\\d+x\\."
enabled: false
strategy: mask
mode: detect
priority: 90
"#;
    let def: RuleDef = serde_yaml::from_str(yaml).unwrap();
    assert!(!def.enabled);
    assert_eq!(def.strategy, Strategy::Mask);
    assert_eq!(def.mode, Mode::Detect);
    assert_eq!(def.priority, 90);
    assert_eq!(def.exclude.unwrap(), "@\\d+x\\.");
}

#[test]
fn test_rule_file_deserialize() {
    let yaml = r#"
version: "1"
name: test_category
description: Test category
rules:
  - id: rule1
    name: Rule 1
    pattern: "\\d+"
  - id: rule2
    name: Rule 2
    pattern: "[a-z]+"
"#;
    let file: RuleFile = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(file.rules.len(), 2);
}

#[test]
fn test_compile_regex_valid() {
    let re = detector::compile_regex(r"\d{11}").unwrap();
    assert!(re.is_match("13812345678"));
}

#[test]
fn test_compile_regex_invalid() {
    assert!(detector::compile_regex(r"[unclosed").is_err());
}

#[test]
fn test_compile_regex_size_limit() {
    let long = "a".repeat(2001);
    assert!(detector::compile_regex(&long).is_err());
}
