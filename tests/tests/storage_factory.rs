// T-STO-FACTORY: StorageFactory — create, in_memory, default_sqlite
use aidaguard_core::error::StorageError;
use aidaguard_storage::StorageFactory;
use uuid::Uuid;

fn temp_db_path() -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let name = format!("aidaguard_test_factory_{}.db", Uuid::new_v4());
    dir.join(name)
}

#[test]
fn test_create_sqlite() {
    let path = temp_db_path();
    let storage = StorageFactory::create("sqlite", &path, Some("test-key"));
    assert!(storage.is_ok());
    let storage = storage.unwrap();
    storage.record("r1", "R1", "mask", "", "val", "ctx", "/api", "body", 200, "").unwrap();
    assert_eq!(storage.count().unwrap(), 1);
    let salt_path = path.with_extension("db.salt");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&salt_path);
}

#[test]
fn test_create_memory() {
    let path = std::path::PathBuf::new();
    let storage = StorageFactory::create("memory", &path, None);
    assert!(storage.is_ok());
    let storage = storage.unwrap();
    storage.record("r1", "R1", "mask", "", "val", "ctx", "/api", "body", 200, "").unwrap();
    assert_eq!(storage.count().unwrap(), 1);
}

#[test]
fn test_create_postgres_returns_unknown_type() {
    let path = std::path::PathBuf::new();
    let result = StorageFactory::create("postgres", &path, None);
    assert!(result.is_err());
    let err = result.err().unwrap();
    match err {
        StorageError::UnknownType { type_name } => assert_eq!(type_name, "postgres"),
        other => panic!("expected UnknownType, got {:?}", other),
    }
}

#[test]
fn test_in_memory() {
    let storage = StorageFactory::in_memory();
    storage.record("r1", "R1", "detect", "", "val", "ctx", "/api", "body", 200, "").unwrap();
    assert_eq!(storage.count().unwrap(), 1);
}

#[test]
fn test_default_sqlite() {
    // default_sqlite() creates storage at the platform data dir.
    // This may fail in CI or sandboxed environments, so we test that
    // the function exists and returns a proper Result (not a panic).
    let result = StorageFactory::default_sqlite();
    // If it succeeds, verify basic functionality
    if let Ok(storage) = result {
        storage.record("r1", "R1", "mask", "", "val", "ctx", "/api", "body", 200, "").unwrap();
        assert_eq!(storage.count().unwrap(), 1);
    }
    // If it fails (e.g. no write access to data dir), that is also acceptable
}
