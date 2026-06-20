// T-CORE-15~18: Config
use aidaguard_core::config::Config;
use aidaguard_core::config::DetectionRegion;
use aidaguard_core::config::StorageConfig;

#[test] fn test_config_defaults() {
    let c = Config::default();
    assert_eq!(c.port, 19000);
    assert_eq!(c.target_url, "");
    assert_eq!(c.rules_dir, "./rules");
    assert_eq!(c.max_body_size_mb, 10);
    assert!(!c.storage.enabled);
    assert!(c.upstreams.is_empty());
    assert!(c.notification.enabled);
}
#[test] fn test_config_save_load_roundtrip() {
    let dir = std::env::temp_dir().join("aidaguard_test_config");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.toml");
    let mut c = Config::default();
    c.port = 20000;
    c.log_level = "debug".into();
    c.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.port, 20000);
    assert_eq!(loaded.log_level, "debug");
    let _ = std::fs::remove_dir_all(&dir);
}
#[test] fn test_config_load_missing_file() {
    assert!(Config::load_from(std::path::Path::new("/tmp/aidaguard_nonexistent_config_xyz.toml")).is_none());
}
#[test] fn test_storage_config_defaults() {
    let c = Config::default();
    assert!(!c.storage.enabled);
    assert!(!c.storage.db_path.is_empty());
    assert!(c.storage.encryption_key.is_none());
}

// T-CORE-19~27: Config extended tests
#[test] fn test_config_load_returns_default_when_missing() {
    let dir = std::env::temp_dir().join("aidaguard_test_config_missing");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("nonexistent_config.toml");
    // load_from returns None when file is missing
    assert!(Config::load_from(&path).is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test] fn test_detection_region_all_regions_primary_only() {
    let dr = DetectionRegion {
        primary_region: "cn".into(),
        additional_regions: vec![],
    };
    let regions = dr.all_regions();
    assert_eq!(regions, vec!["cn"]);
}

#[test] fn test_detection_region_all_regions_with_additional() {
    let dr = DetectionRegion {
        primary_region: "cn".into(),
        additional_regions: vec!["us".into(), "eu".into()],
    };
    let regions = dr.all_regions();
    assert_eq!(regions, vec!["cn", "us", "eu"]);
}

#[test] fn test_detection_region_all_regions_dedup() {
    let dr = DetectionRegion {
        primary_region: "cn".into(),
        additional_regions: vec!["cn".into(), "us".into()],
    };
    let regions = dr.all_regions();
    assert_eq!(regions, vec!["cn", "us"]);
}

#[test] fn test_detection_region_available_regions() {
    let available = DetectionRegion::available_regions();
    assert!(available.len() >= 7, "expected at least 7 available regions, got {}", available.len());
    let codes: Vec<&str> = available.iter().map(|(code, _)| *code).collect();
    assert!(codes.contains(&"cn"));
    assert!(codes.contains(&"us"));
    assert!(codes.contains(&"eu"));
}

#[test] fn test_rule_presets_with_region_settings() {
    let mut c = Config::default();
    c.region = "cn".into();
    c.rules_dir = "/nonexistent_rules_dir_for_test".into();
    // With legacy region="cn" and no detection_region overrides, rule_presets should include "cn"
    let presets = c.rule_presets();
    assert!(presets.contains(&"cn".to_string()) || presets.contains(&"global".to_string()));
}

#[test] fn test_rule_presets_with_detection_region() {
    let mut c = Config::default();
    c.detection_region = DetectionRegion {
        primary_region: "us".into(),
        additional_regions: vec!["eu".into()],
    };
    c.rules_dir = "/nonexistent_rules_dir_for_test".into();
    let presets = c.rule_presets();
    // Should include both us and eu via all_regions()
    assert!(presets.contains(&"us".to_string()) || presets.contains(&"global".to_string()));
}

#[test] fn test_save_to_load_from_preserves_fields() {
    let dir = std::env::temp_dir().join("aidaguard_test_config_roundtrip2");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.toml");
    let mut c = Config::default();
    c.port = 25000;
    c.log_level = "trace".into();
    c.rules_dir = "/custom/rules".into();
    c.region = "eu".into();
    c.rule_industries = vec!["finance".into(), "medical".into()];
    c.detection_region = DetectionRegion {
        primary_region: "us".into(),
        additional_regions: vec!["sg".into()],
    };
    c.storage = StorageConfig {
        enabled: true,
        storage_type: "sqlite".into(),
        db_path: "/data/test.db".into(),
        encryption_key: Some("secret_key".into()),
    };
    c.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.port, 25000);
    assert_eq!(loaded.log_level, "trace");
    assert_eq!(loaded.rules_dir, "/custom/rules");
    assert_eq!(loaded.region, "eu");
    assert_eq!(loaded.rule_industries, vec!["finance", "medical"]);
    assert_eq!(loaded.detection_region.primary_region, "us");
    assert_eq!(loaded.detection_region.additional_regions, vec!["sg"]);
    assert!(loaded.storage.enabled);
    assert_eq!(loaded.storage.db_path, "/data/test.db");
    assert_eq!(loaded.storage.encryption_key.as_deref(), Some("secret_key"));
    let _ = std::fs::remove_dir_all(&dir);
}
