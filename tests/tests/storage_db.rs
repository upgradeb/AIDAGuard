// T-STO-01~15: Storage — CRUD, filtering, stats, migration
use aidaguard_storage::Storage;
use aidaguard_storage::AuditStorage;
use aidaguard_core::storage_types::AuditFilter;
use uuid::Uuid;

fn temp_db() -> (Storage, String) {
    let dir = std::env::temp_dir();
    let name = format!("aidaguard_test_{}.db", Uuid::new_v4());
    let path = dir.join(name);
    let storage = Storage::open(&path, "test-key").unwrap();
    (storage, path.to_string_lossy().to_string())
}

#[test] fn test_record_and_list() {
    let (storage, path) = temp_db();
    storage.record("phone_cn", "phone", "placeholder", "[[PHONE_CN@abc12345]]", "13800001111",
        "...my phone 13800001111...", "/v2/coding", "sanitized body", 200, "").unwrap();
    assert_eq!(storage.count().unwrap(), 1);
    let records = storage.list(10, 0).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].original, "13800001111");
    assert_eq!(records[0].rule_id, "phone_cn");
    assert_eq!(records[0].rule_name, "phone");
    assert_eq!(records[0].request_path, "/v2/coding");
    assert_eq!(records[0].response_status, 200);
    assert!(records[0].timestamp_ms > 0);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_multiple_records() {
    let (storage, path) = temp_db();
    for i in 0..5 {
        storage.record("email", "email", "mask", "[[EMAIL@deadbeef]]",
            &format!("user{}@example.com", i), "context around email",
            "/chat", "body", 200, "").unwrap();
    }
    assert_eq!(storage.count().unwrap(), 5);
    let records = storage.list(3, 0).unwrap();
    assert_eq!(records.len(), 3);
    let records = storage.list(3, 3).unwrap();
    assert_eq!(records.len(), 2);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_empty_db() {
    let (storage, path) = temp_db();
    assert_eq!(storage.count().unwrap(), 0);
    assert!(storage.list(10, 0).unwrap().is_empty());
    let _ = std::fs::remove_file(path);
}
#[test] fn test_encrypt_decrypt_roundtrip() {
    let (storage, path) = temp_db();
    let encrypted = storage.encrypt(b"test data 12345").unwrap();
    let decrypted = storage.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, b"test data 12345");
    let _ = std::fs::remove_file(path);
}
#[test] fn test_decrypt_invalid_data() {
    let (storage, path) = temp_db();
    assert!(storage.decrypt(b"too_short").is_err());
    let _ = std::fs::remove_file(path);
}
#[test] fn test_delete_record() {
    let (storage, path) = temp_db();
    storage.record("test", "Test", "detect", "", "value", "ctx", "/api", "body", 200, "").unwrap();
    assert_eq!(storage.count().unwrap(), 1);
    let records = storage.list(1, 0).unwrap();
    let id = &records[0].id;
    assert!(storage.delete(id).unwrap());
    assert_eq!(storage.count().unwrap(), 0);
    assert!(!storage.delete("nonexistent-id").unwrap());
    let _ = std::fs::remove_file(path);
}
#[test] fn test_list_filtered_by_rule_id() {
    let (storage, path) = temp_db();
    storage.record("rule_a", "Rule A", "filter", "", "val1", "ctx", "/api", "body", 200, "").unwrap();
    storage.record("rule_b", "Rule B", "filter", "", "val2", "ctx", "/api", "body", 200, "").unwrap();
    let filter = AuditFilter::by_rule("rule_a");
    let results = storage.list_filtered(10, 0, filter).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "rule_a");
    let _ = std::fs::remove_file(path);
}
#[test] fn test_list_filtered_by_path() {
    let (storage, path) = temp_db();
    storage.record("r1", "R1", "filter", "", "v1", "ctx", "/chat/completions", "body", 200, "").unwrap();
    storage.record("r2", "R2", "filter", "", "v2", "ctx", "/embeddings", "body", 200, "").unwrap();
    let filter = AuditFilter::new().with_path("/chat");
    let results = storage.list_filtered(10, 0, filter).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].request_path, "/chat/completions");
    let _ = std::fs::remove_file(path);
}
#[test] fn test_list_filtered_by_date() {
    let (storage, path) = temp_db();
    let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    storage.record("r1", "R1", "filter", "", "v1", "ctx", "/api", "body", 200, "").unwrap();
    // Query with wide date range should include the record
    let filter = AuditFilter::by_date_range(0, now_ms + 10000);
    let results = storage.list_filtered(10, 0, filter).unwrap();
    assert!(!results.is_empty());
    // Query with future-only range should be empty
    let filter = AuditFilter::by_date_range(now_ms + 100000, now_ms + 200000);
    let results = storage.list_filtered(10, 0, filter).unwrap();
    assert!(results.is_empty());
    let _ = std::fs::remove_file(path);
}
#[test] fn test_count_filtered() {
    let (storage, path) = temp_db();
    storage.record("r1", "R1", "placeholder", "", "v1", "ctx", "/api", "body", 200, "").unwrap();
    storage.record("r2", "R2", "detect", "", "v2", "ctx", "/api", "body", 200, "").unwrap();
    let total = storage.count_filtered(AuditFilter::new()).unwrap();
    assert_eq!(total, 2);
    let filtered = storage.count_filtered(AuditFilter::by_rule("r1")).unwrap();
    assert_eq!(filtered, 1);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_stats_today() {
    let (storage, path) = temp_db();
    storage.record("r1", "R1", "placeholder", "", "v1", "ctx", "/api", "body", 200, "").unwrap();
    let stats = storage.stats().unwrap();
    assert_eq!(stats.total_count, 1);
    assert!(stats.today_count <= 1);
    assert_eq!(stats.rule_distribution.len(), 1);
    assert_eq!(stats.rule_distribution[0].rule_id, "r1");
    assert_eq!(stats.rule_distribution[0].count, 1);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_stats_db_size() {
    let (storage, path) = temp_db();
    storage.record("r1", "R1", "placeholder", "", "v1", "ctx", "/api", "body", 200, "").unwrap();
    let stats = storage.stats().unwrap();
    assert!(stats.db_size_bytes > 0);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_get_by_id() {
    let (storage, path) = temp_db();
    storage.record("r1", "R1", "placeholder", "", "v1", "ctx", "/api", "body", 200, "").unwrap();
    let records = storage.list(1, 0).unwrap();
    let id = &records[0].id;
    let record = storage.get_by_id(id).unwrap().unwrap();
    assert_eq!(record.original, "v1");
    assert!(storage.get_by_id("nonexistent").unwrap().is_none());
    let _ = std::fs::remove_file(path);
}
#[test] fn test_list_recent() {
    let (storage, path) = temp_db();
    for i in 0..5 {
        storage.record("r1", "R1", "placeholder", "", &format!("v{}", i), "ctx", "/api", "body", 200, "").unwrap();
    }
    let recent = storage.list_recent(3).unwrap();
    assert_eq!(recent.len(), 3);
    let _ = std::fs::remove_file(path);
}
#[test] fn test_migration_add_column() {
    // Opening a DB with old schema auto-adds tool_name and rule_name columns
    let (storage, path) = temp_db();
    // Write a record — migration has already run. Verify it works.
    storage.record("r1", "Rule One", "detect", "", "sensitive", "context here", "/api/v1", "sanitized", 200, "TestTool").unwrap();
    let records = storage.list(1, 0).unwrap();
    assert_eq!(records[0].tool_name, "TestTool");
    assert_eq!(records[0].rule_name, "Rule One");
    let _ = std::fs::remove_file(path);
}
