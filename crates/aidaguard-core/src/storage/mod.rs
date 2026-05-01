use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;
use uuid::Uuid;

/// 一条检测审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRecord {
    pub id: String,
    pub timestamp_ms: i64,
    pub rule_id: String,
    pub strategy: String,
    pub placeholder: String,
    pub original: String,
    pub context: String,
    pub request_path: String,
    pub sanitized_body: String,
    pub response_status: u16,
}

/// 加密持久化存储
///
/// SQLite + AES-256-GCM，用于审计记录敏感数据检测历史。
/// `original` 字段在写入前加密，读取时解密。
pub struct Storage {
    conn: Mutex<Connection>,
    cipher: Aes256Gcm,
}

impl Storage {
    /// 打开（或创建）数据库。
    ///
    /// `encryption_key` 为任意长度的密钥字符串，内部使用 SHA-256 派生 32 字节 AES 密钥。
    pub fn open(db_path: &Path, encryption_key: &str) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS detections (
                id          TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL,
                rule_id     TEXT NOT NULL,
                strategy    TEXT NOT NULL,
                placeholder TEXT NOT NULL,
                original_encrypted BLOB NOT NULL,
                context_encrypted BLOB NOT NULL DEFAULT (x''),
                request_path TEXT NOT NULL DEFAULT '',
                sanitized_body TEXT NOT NULL DEFAULT '',
                response_status INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_detections_timestamp
                ON detections(timestamp_ms DESC);
            ",
        )?;

        let mut hasher = Sha256::new();
        hasher.update(encryption_key.as_bytes());
        let key_bytes = hasher.finalize();
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        tracing::info!("Storage opened: {}", db_path.display());

        Ok(Self {
            conn: Mutex::new(conn),
            cipher,
        })
    }

    /// 记录一条检测结果。`original` 和 `context` 写入前被加密。
    pub fn record(
        &self,
        rule_id: &str,
        strategy: &str,
        placeholder: &str,
        original: &str,
        context: &str,
        request_path: &str,
        sanitized_body: &str,
        response_status: u16,
    ) -> Result<(), anyhow::Error> {
        let id = Uuid::new_v4().to_string();
        let timestamp_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let encrypted_original = self.encrypt(original.as_bytes())?;
        let encrypted_context = self.encrypt(context.as_bytes())?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO detections (id, timestamp_ms, rule_id, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![id, timestamp_ms, rule_id, strategy, placeholder, encrypted_original, encrypted_context, request_path, sanitized_body, response_status],
        )?;

        tracing::debug!("detection recorded: {} -> {}", rule_id, placeholder);
        Ok(())
    }

    /// 分页查询检测记录，按时间倒序，返回时解密 `original` 和 `context` 字段。
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, rule_id, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status
             FROM detections ORDER BY timestamp_ms DESC LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit as i64, offset as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Vec<u8>>(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i64>(9)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (id, timestamp_ms, rule_id, strategy, placeholder, enc_original, enc_context, request_path, sanitized_body, response_status) = row?;
            let original_bytes = self.decrypt(&enc_original)?;
            let original = String::from_utf8(original_bytes)
                .unwrap_or_else(|_| "<非 UTF-8 数据>".to_string());
            let context_bytes = self.decrypt(&enc_context).unwrap_or_default();
            let context = String::from_utf8(context_bytes)
                .unwrap_or_else(|_| "<非 UTF-8 数据>".to_string());

            records.push(DetectionRecord {
                id,
                timestamp_ms,
                rule_id,
                strategy,
                placeholder,
                original,
                context,
                request_path,
                sanitized_body,
                response_status: response_status as u16,
            });
        }

        Ok(records)
    }

    /// 检测记录总数
    pub fn count(&self) -> Result<usize, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM detections", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
        let mut out = nonce.to_vec();
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("invalid encrypted data"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (Storage, String) {
        let dir = std::env::temp_dir();
        let name = format!("aidaguard_test_{}.db", Uuid::new_v4());
        let path = dir.join(name);
        let storage = Storage::open(&path, "test-key").unwrap();
        (storage, path.to_string_lossy().to_string())
    }

    #[test]
    fn test_record_and_list() {
        let (storage, path) = temp_db();

        storage
            .record("phone_cn", "placeholder", "[[PHONE_CN@abc12345]]", "13800001111", "...我的手机号13800001111，请记录...", "/v2/coding", "sanitized body", 200)
            .unwrap();

        assert_eq!(storage.count().unwrap(), 1);

        let records = storage.list(10, 0).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].original, "13800001111");
        assert_eq!(records[0].context, "...我的手机号13800001111，请记录...");
        assert_eq!(records[0].rule_id, "phone_cn");
        assert_eq!(records[0].request_path, "/v2/coding");
        assert_eq!(records[0].sanitized_body, "sanitized body");
        assert_eq!(records[0].response_status, 200);
        assert!(records[0].timestamp_ms > 0);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_multiple_records() {
        let (storage, path) = temp_db();

        for i in 0..5 {
            storage
                .record(
                    "email",
                    "mask",
                    "[[EMAIL@deadbeef]]",
                    &format!("user{}@example.com", i),
                    "context around email",
                    "/chat",
                    "body",
                    200,
                )
                .unwrap();
        }

        assert_eq!(storage.count().unwrap(), 5);

        let records = storage.list(3, 0).unwrap();
        assert_eq!(records.len(), 3);

        let records = storage.list(3, 3).unwrap();
        assert_eq!(records.len(), 2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_empty_db() {
        let (storage, path) = temp_db();
        assert_eq!(storage.count().unwrap(), 0);
        assert!(storage.list(10, 0).unwrap().is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (storage, path) = temp_db();
        let encrypted = storage.encrypt(b"test data 12345").unwrap();
        let decrypted = storage.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"test data 12345");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let (storage, path) = temp_db();
        assert!(storage.decrypt(b"too_short").is_err());
        let _ = std::fs::remove_file(path);
    }
}
