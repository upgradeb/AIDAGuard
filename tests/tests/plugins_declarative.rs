// T-PLG-20~34: Declarative adapter — WriteValue, DetectConfig, DeclarativeAdapter, load_all
use aidaguard_plugins::declarative::{
    DeclarativeAdapter, ToolManifest, load_all,
    manifest::{WriteValue, DetectConfig, FileConfig, ReadWriteConfig, ConfigFormat, RestoreMode},
};
use aidaguard_plugins::ToolAdapter;
use std::collections::HashMap;

fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("aidaguard_test_declarative_{}", uuid::Uuid::new_v4()))
}

// -- WriteValue tests --

#[test]
fn test_write_value_resolve_proxy_url() {
    let wv = WriteValue::ProxyUrl;
    assert_eq!(wv.resolve("http://localhost:19000"), Some("http://localhost:19000".to_string()));
}

#[test]
fn test_write_value_resolve_static() {
    let wv = WriteValue::Static("fixed".to_string());
    assert_eq!(wv.resolve("irrelevant"), Some("fixed".to_string()));
}

#[test]
fn test_write_value_resolve_delete() {
    let wv = WriteValue::Delete;
    assert_eq!(wv.resolve("irrelevant"), None);
}

#[test]
fn test_write_value_is_proxy_url() {
    assert!(WriteValue::ProxyUrl.is_proxy_url());
    assert!(!WriteValue::Static("x".to_string()).is_proxy_url());
    assert!(!WriteValue::Delete.is_proxy_url());
}

#[test]
fn test_write_value_deserialize_proxy_url() {
    let wv: WriteValue = serde_json::from_str(r#""proxyUrl""#).unwrap();
    assert!(wv.is_proxy_url());
}

#[test]
fn test_write_value_deserialize_static() {
    let wv: WriteValue = serde_json::from_str(r#""http://example.com""#).unwrap();
    match wv {
        WriteValue::Static(s) => assert_eq!(s, "http://example.com"),
        _ => panic!("expected Static"),
    }
}

#[test]
fn test_write_value_deserialize_delete() {
    let wv: WriteValue = serde_json::from_str(r#""""#).unwrap();
    matches!(wv, WriteValue::Delete);
}

// -- DetectConfig tests via DeclarativeAdapter --

#[test]
fn test_detect_config_dir_exists() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = ToolManifest {
        id: "test_dir".into(), name: "Test Dir".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::DirExists { path: dir.to_string_lossy().to_string() },
        config: None, secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(adapter.detect());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_config_file_exists() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("config.json");
    std::fs::write(&file_path, "{}").unwrap();
    let manifest = ToolManifest {
        id: "test_file".into(), name: "Test File".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::FileExists { path: file_path.to_string_lossy().to_string() },
        config: None, secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(adapter.detect());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_detect_config_always() {
    let manifest = ToolManifest {
        id: "test_always".into(), name: "Always".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::Always,
        config: None, secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(adapter.detect());
}

#[test]
fn test_detect_config_missing_path() {
    let manifest = ToolManifest {
        id: "test_missing".into(), name: "Missing".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::FileExists { path: "/nonexistent/path/config.json".into() },
        config: None, secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(!adapter.detect());
}

// -- load_all tests --

#[test]
fn test_load_all_loads_manifests() {
    let plugins = load_all();
    assert!(plugins.len() > 0, "load_all should return at least one plugin");
}

#[test]
fn test_declarative_adapter_manifest() {
    let plugins = load_all();
    let first = &plugins[0];
    let m = first.manifest();
    assert!(!m.id.is_empty());
    assert!(!m.name.is_empty());
}

#[test]
fn test_declarative_adapter_configure_and_detect() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("test_config.json");
    std::fs::write(&config_path, r#"{"endpoint":"https://api.openai.com"}"#).unwrap();

    let mut write_map: HashMap<String, WriteValue> = HashMap::new();
    write_map.insert("endpoint".to_string(), WriteValue::ProxyUrl);

    let manifest = ToolManifest {
        id: "test_cfg".into(), name: "Test Cfg".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::Always,
        config: Some(FileConfig {
            path: config_path.to_string_lossy().to_string(),
            format: ConfigFormat::Json,
            endpoint: Some(ReadWriteConfig {
                read: vec!["endpoint".to_string()],
                write: write_map,
                read_env_fallback: None,
            }),
            model: None,
            restore_mode: RestoreMode::File,
        }),
        secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);

    let backup_dir = dir.join("backups");
    adapter.backup(&backup_dir).unwrap();
    adapter.configure("http://127.0.0.1:19000").unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("127.0.0.1"), "configured endpoint should contain 127.0.0.1");

    adapter.restore(&backup_dir).unwrap();
    let restored = std::fs::read_to_string(&config_path).unwrap();
    assert!(restored.contains("api.openai.com"), "restored endpoint should contain original value");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_declarative_adapter_is_configured_local() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("local_config.json");
    std::fs::write(&config_path, r#"{"endpoint":"http://127.0.0.1:19000"}"#).unwrap();

    let manifest = ToolManifest {
        id: "test_local".into(), name: "Local".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::Always,
        config: Some(FileConfig {
            path: config_path.to_string_lossy().to_string(),
            format: ConfigFormat::Json,
            endpoint: Some(ReadWriteConfig {
                read: vec!["endpoint".to_string()],
                write: HashMap::new(),
                read_env_fallback: None,
            }),
            model: None,
            restore_mode: RestoreMode::File,
        }),
        secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(adapter.is_configured());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_declarative_adapter_is_configured_remote() {
    let dir = temp_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let config_path = dir.join("remote_config.json");
    std::fs::write(&config_path, r#"{"endpoint":"https://api.openai.com"}"#).unwrap();

    let manifest = ToolManifest {
        id: "test_remote".into(), name: "Remote".into(),
        version: "1.0".into(), description: "test".into(), author: "test".into(),
        categories: vec!["test".into()],
        detect: DetectConfig::Always,
        config: Some(FileConfig {
            path: config_path.to_string_lossy().to_string(),
            format: ConfigFormat::Json,
            endpoint: Some(ReadWriteConfig {
                read: vec!["endpoint".to_string()],
                write: HashMap::new(),
                read_env_fallback: None,
            }),
            model: None,
            restore_mode: RestoreMode::File,
        }),
        secondary_configs: vec![], custom: false,
    };
    let adapter = DeclarativeAdapter::new(manifest);
    assert!(!adapter.is_configured());
    let _ = std::fs::remove_dir_all(&dir);
}
