// T-CORE-15~18: Config
use aidaguard_core::config::Config;

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
