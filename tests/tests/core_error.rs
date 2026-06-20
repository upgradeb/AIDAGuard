// T-CORE-ERR-01~23: Error types
use aidaguard_core::error::*;

#[test] fn test_detection_error_codes() {
    assert_eq!(DetectionError::RuleCompilation("bad".into()).code(), "DET_001");
    assert_eq!(DetectionError::InvalidRegex { pattern: "(".into(), reason: "unclosed".into() }.code(), "DET_002");
    assert_eq!(DetectionError::RuleNotFound("r1".into()).code(), "DET_003");
    assert_eq!(DetectionError::RuleFileParse { path: "f.yaml".into(), reason: "bad".into() }.code(), "DET_004");
    assert_eq!(DetectionError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "gone")).code(), "DET_005");
    assert_eq!(DetectionError::ModelNotLoaded("en".into()).code(), "DET_006");
    assert_eq!(DetectionError::EngineNotInitialized.code(), "DET_007");
    assert_eq!(DetectionError::DetectionTimeout { duration_ms: 5000 }.code(), "DET_008");
    assert_eq!(DetectionError::Config("missing".into()).code(), "DET_009");
}

#[test] fn test_detection_error_recovery_hints() {
    let variants: Vec<DetectionError> = vec![
        DetectionError::RuleCompilation("x".into()),
        DetectionError::InvalidRegex { pattern: "x".into(), reason: "x".into() },
        DetectionError::RuleNotFound("x".into()),
        DetectionError::RuleFileParse { path: "x".into(), reason: "x".into() },
        DetectionError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x")),
        DetectionError::ModelNotLoaded("x".into()),
        DetectionError::EngineNotInitialized,
        DetectionError::DetectionTimeout { duration_ms: 1 },
        DetectionError::Config("x".into()),
    ];
    for v in &variants {
        assert!(!v.recovery_hint().is_empty(), "recovery_hint should not be empty for {:?}", v);
    }
}

#[test] fn test_storage_error_codes() {
    assert_eq!(StorageError::ConnectionFailed { path: "db".into(), reason: "timeout".into() }.code(), "STO_001");
    assert_eq!(StorageError::DatabaseLocked.code(), "STO_002");
    assert_eq!(StorageError::EncryptionError("bad".into()).code(), "STO_003");
    assert_eq!(StorageError::DecryptionError("bad".into()).code(), "STO_004");
    assert_eq!(StorageError::InvalidKey.code(), "STO_005");
    assert_eq!(StorageError::NotFound("rec".into()).code(), "STO_006");
    assert_eq!(StorageError::MigrationFailed("v2".into()).code(), "STO_007");
    assert_eq!(StorageError::UnknownType { type_name: "csv".into() }.code(), "STO_008");
    assert_eq!(StorageError::MissingConfig { field: "db_path".into() }.code(), "STO_009");
    assert_eq!(StorageError::TooLarge { size: 100, max_size: 50 }.code(), "STO_010");
}

#[test] fn test_storage_error_recovery_hints() {
    let variants: Vec<StorageError> = vec![
        StorageError::ConnectionFailed { path: "x".into(), reason: "x".into() },
        StorageError::DatabaseLocked,
        StorageError::EncryptionError("x".into()),
        StorageError::DecryptionError("x".into()),
        StorageError::InvalidKey,
        StorageError::NotFound("x".into()),
        StorageError::MigrationFailed("x".into()),
        StorageError::UnknownType { type_name: "x".into() },
        StorageError::MissingConfig { field: "x".into() },
        StorageError::TooLarge { size: 1, max_size: 1 },
    ];
    for v in &variants {
        assert!(!v.recovery_hint().is_empty(), "recovery_hint should not be empty for {:?}", v);
    }
}

#[test] fn test_proxy_error_codes() {
    assert_eq!(ProxyError::UpstreamConnection { url: "http://x".into(), reason: "refused".into() }.code(), "PRX_001");
    assert_eq!(ProxyError::Timeout { duration_secs: 30 }.code(), "PRX_002");
    assert_eq!(ProxyError::InvalidBody { reason: "bad json".into() }.code(), "PRX_003");
    assert_eq!(ProxyError::RateLimitExceeded.code(), "PRX_004");
    assert_eq!(ProxyError::UpstreamError { status: 500, message: "err".into() }.code(), "PRX_005");
    assert_eq!(ProxyError::SensitiveDataDetected { count: 3, types: "PHONE".into() }.code(), "PRX_006");
    assert_eq!(ProxyError::StreamError { reason: "eof".into() }.code(), "PRX_007");
}

#[test] fn test_proxy_error_recovery_hints() {
    let variants: Vec<ProxyError> = vec![
        ProxyError::UpstreamConnection { url: "x".into(), reason: "x".into() },
        ProxyError::Timeout { duration_secs: 1 },
        ProxyError::InvalidBody { reason: "x".into() },
        ProxyError::RateLimitExceeded,
        ProxyError::UpstreamError { status: 500, message: "x".into() },
        ProxyError::SensitiveDataDetected { count: 1, types: "x".into() },
        ProxyError::StreamError { reason: "x".into() },
    ];
    for v in &variants {
        assert!(!v.recovery_hint().is_empty(), "recovery_hint should not be empty for {:?}", v);
    }
}

#[test] fn test_config_error_codes() {
    assert_eq!(ConfigError::FileNotFound { path: "f.toml".into() }.code(), "CFG_001");
    assert_eq!(ConfigError::ParseFailed { path: "f.toml".into(), reason: "bad".into() }.code(), "CFG_002");
    assert_eq!(ConfigError::MissingField { field: "port".into() }.code(), "CFG_003");
    assert_eq!(ConfigError::InvalidValue { field: "port".into(), value: "-1".into(), reason: "negative".into() }.code(), "CFG_004");
}

#[test] fn test_config_error_recovery_hints() {
    let variants: Vec<ConfigError> = vec![
        ConfigError::FileNotFound { path: "x".into() },
        ConfigError::ParseFailed { path: "x".into(), reason: "x".into() },
        ConfigError::MissingField { field: "x".into() },
        ConfigError::InvalidValue { field: "x".into(), value: "x".into(), reason: "x".into() },
    ];
    for v in &variants {
        assert!(!v.recovery_hint().is_empty(), "recovery_hint should not be empty for {:?}", v);
    }
}

#[test] fn test_plugin_error_codes() {
    assert_eq!(PluginError::LoadFailed { id: "p1".into(), reason: "bad".into() }.code(), "PLG_001");
    assert_eq!(PluginError::NotFound { id: "p1".into() }.code(), "PLG_002");
    assert_eq!(PluginError::ConfigureFailed { id: "p1".into(), reason: "bad".into() }.code(), "PLG_003");
    assert_eq!(PluginError::RestoreFailed { id: "p1".into() }.code(), "PLG_004");
    assert_eq!(PluginError::AbiMismatch { expected: 1, actual: 2 }.code(), "PLG_005");
    assert_eq!(PluginError::LibraryNotFound { dir: "/plugins".into() }.code(), "PLG_006");
    assert_eq!(PluginError::InvalidSignature { id: "p1".into() }.code(), "PLG_007");
    assert_eq!(PluginError::InvalidManifest { reason: "bad".into() }.code(), "PLG_008");
    assert_eq!(PluginError::InitFailed { id: "p1".into() }.code(), "PLG_009");
}

#[test] fn test_plugin_error_recovery_hints() {
    let variants: Vec<PluginError> = vec![
        PluginError::LoadFailed { id: "x".into(), reason: "x".into() },
        PluginError::NotFound { id: "x".into() },
        PluginError::ConfigureFailed { id: "x".into(), reason: "x".into() },
        PluginError::RestoreFailed { id: "x".into() },
        PluginError::AbiMismatch { expected: 1, actual: 2 },
        PluginError::LibraryNotFound { dir: "x".into() },
        PluginError::InvalidSignature { id: "x".into() },
        PluginError::InvalidManifest { reason: "x".into() },
        PluginError::InitFailed { id: "x".into() },
    ];
    for v in &variants {
        assert!(!v.recovery_hint().is_empty(), "recovery_hint should not be empty for {:?}", v);
    }
}

#[test] fn test_aidaguard_error_delegation_code() {
    let err = AidaGuardError::Detection(DetectionError::RuleCompilation("bad".into()));
    assert_eq!(err.code(), "DET_001");
}

#[test] fn test_aidaguard_error_user_message_detection() {
    let err = AidaGuardError::Detection(DetectionError::RuleCompilation("bad".into()));
    assert!(err.user_message().contains("检测错误"));
}

#[test] fn test_aidaguard_error_user_message_storage() {
    let err = AidaGuardError::Storage(StorageError::DatabaseLocked);
    assert!(err.user_message().contains("存储错误"));
}

#[test] fn test_aidaguard_error_user_message_proxy() {
    let err = AidaGuardError::Proxy(ProxyError::RateLimitExceeded);
    assert!(err.user_message().contains("代理错误"));
}

#[test] fn test_aidaguard_error_user_message_config() {
    let err = AidaGuardError::Config(ConfigError::FileNotFound { path: "x".into() });
    assert!(err.user_message().contains("配置错误"));
}

#[test] fn test_aidaguard_error_user_message_plugin() {
    let err = AidaGuardError::Plugin(PluginError::NotFound { id: "x".into() });
    assert!(err.user_message().contains("插件错误"));
}

#[test] fn test_aidaguard_error_recovery_hint_delegation() {
    let err = AidaGuardError::Detection(DetectionError::EngineNotInitialized);
    assert_eq!(err.recovery_hint(), DetectionError::EngineNotInitialized.recovery_hint());
}

#[test] fn test_error_response_from_aidaguard_error() {
    let err = AidaGuardError::Detection(DetectionError::RuleCompilation("bad".into()));
    let resp: ErrorResponse = (&err).into();
    assert_eq!(resp.code, "DET_001");
    assert!(!resp.message.is_empty());
    assert!(!resp.recovery_hint.is_empty());
    assert!(resp.details.is_some());
}

#[test] fn test_error_response_from_detection_error() {
    let err = DetectionError::RuleCompilation("bad".into());
    let resp: ErrorResponse = (&err).into();
    assert_eq!(resp.code, "DET_001");
    assert!(resp.message.contains("检测错误"));
}

#[test] fn test_error_response_from_storage_error() {
    let err = StorageError::DatabaseLocked;
    let resp: ErrorResponse = (&err).into();
    assert_eq!(resp.code, "STO_002");
    assert!(resp.message.contains("存储错误"));
}

#[test] fn test_error_response_from_proxy_error() {
    let err = ProxyError::RateLimitExceeded;
    let resp: ErrorResponse = (&err).into();
    assert_eq!(resp.code, "PRX_004");
    assert!(resp.message.contains("代理错误"));
}

#[test] fn test_aidaguard_error_from_conversion() {
    let det: DetectionError = DetectionError::RuleCompilation("bad".into());
    let err: AidaGuardError = det.into();
    matches!(err, AidaGuardError::Detection(_));
}

#[test] fn test_error_display_messages() {
    let det = DetectionError::RuleCompilation("bad".into());
    assert!(!format!("{}", det).is_empty());
    let sto = StorageError::ConnectionFailed { path: "test.db".into(), reason: "timeout".into() };
    assert!(!format!("{}", sto).is_empty());
    let prx = ProxyError::UpstreamConnection { url: "http://test".into(), reason: "refused".into() };
    assert!(!format!("{}", prx).is_empty());
    let cfg = ConfigError::FileNotFound { path: "test.toml".into() };
    assert!(!format!("{}", cfg).is_empty());
    let plg = PluginError::LoadFailed { id: "test".into(), reason: "bad".into() };
    assert!(!format!("{}", plg).is_empty());
}
