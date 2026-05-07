use std::path::Path;

use aidaguard_core::detector::Detector;

fn count_enabled_yaml_rules(dir: &Path) -> usize {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += count_enabled_yaml_rules(&path);
            } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(rf) = serde_yaml::from_str::<aidaguard_core::detector::RuleFile>(&contents) {
                        total += rf.rules.into_iter().filter(|r| r.enabled).count();
                    }
                }
            }
        }
    }
    total
}

#[test]
fn all_enabled_rules_compile_and_load() {
    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rules");

    assert!(rules_dir.exists(), "rules directory should exist");

    let total_enabled = count_enabled_yaml_rules(&rules_dir);
    assert!(total_enabled >= 30, "Expected 30+ enabled rules, got {}", total_enabled);

    let mut detector = Detector::new();
    let compiled = detector
        .load_from_dir(&rules_dir)
        .expect("should load rules from directory");

    assert_eq!(
        compiled, total_enabled,
        "Expected all {} enabled rules to compile, but only {} did. \
         Check for unsupported regex features (lookahead/lookbehind).",
        total_enabled, compiled
    );

    assert_eq!(detector.rule_count(), compiled);
}

#[test]
fn detect_with_loaded_rules() {
    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rules");

    let mut detector = Detector::new();
    detector.load_from_dir(&rules_dir).unwrap();

    // Phone number detection
    let matches = detector.detect("我的手机号是13812345678");
    assert!(matches.iter().any(|m| m.rule_id == "phone_cn"), "Should detect phone");

    // Email detection
    let matches = detector.detect("contact test@example.com for info");
    assert!(matches.iter().any(|m| m.rule_id == "email"), "Should detect email");

    // ID card detection
    let matches = detector.detect("身份证号110101199003076632");
    assert!(matches.iter().any(|m| m.rule_id == "id_card_cn"), "Should detect ID card");

    // URL detection
    let matches = detector.detect("visit https://example.com/path");
    assert!(matches.iter().any(|m| m.rule_id == "url"), "Should detect URL");

    // JWT detection
    let matches = detector.detect("Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U");
    assert!(matches.iter().any(|m| m.rule_id == "jwt_token"), "Should detect JWT");
}
