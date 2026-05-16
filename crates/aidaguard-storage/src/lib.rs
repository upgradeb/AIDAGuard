use anyhow::{Context, Result};
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, KeyInit};
use rusqlite::Connection;
use sha2::Sha256;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;
use uuid::Uuid;

// Re-export types from core for backward compatibility
pub use aidaguard_core::storage_types::{
    AuditFilter, AuditGroup, AuditStats, DetectionRecord, RuleCount,
};
pub use aidaguard_core::storage_trait::AuditStorage;
pub use aidaguard_core::error::StorageError;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 16;

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
    /// `encryption_key` 为任意长度的密钥字符串，内部使用 PBKDF2-HMAC-SHA256
    /// (600,000 迭代) + 随机 salt 派生 32 字节 AES-256 密钥。
    ///
    /// 启用 WAL 模式以提升并发写入性能。
    pub fn open(db_path: &Path, encryption_key: &str) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(db_path)?;

        // 启用 WAL 模式和性能优化 PRAGMA
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=-64000;  -- 64MB cache
             PRAGMA busy_timeout=5000;
             PRAGMA temp_store=MEMORY;",
        )?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS detections (
                id          TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL,
                rule_id     TEXT NOT NULL,
                rule_name   TEXT NOT NULL DEFAULT '',
                strategy    TEXT NOT NULL,
                placeholder TEXT NOT NULL,
                original_encrypted BLOB NOT NULL,
                context_encrypted BLOB NOT NULL DEFAULT (x''),
                request_path TEXT NOT NULL DEFAULT '',
                sanitized_body TEXT NOT NULL DEFAULT '',
                response_status INTEGER NOT NULL DEFAULT 0,
                tool_name   TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_detections_timestamp
                ON detections(timestamp_ms DESC);
            ",
        )?;

        // 向前兼容：为旧数据库添加 tool_name 列
        let _ = conn.execute_batch(
            "ALTER TABLE detections ADD COLUMN tool_name TEXT NOT NULL DEFAULT '';",
        );
        // 向前兼容：为旧数据库添加 rule_name 列
        let _ = conn.execute_batch(
            "ALTER TABLE detections ADD COLUMN rule_name TEXT NOT NULL DEFAULT '';",
        );

        let cipher = derive_cipher(db_path, encryption_key)?;

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
        rule_name: &str,
        strategy: &str,
        placeholder: &str,
        original: &str,
        context: &str,
        request_path: &str,
        sanitized_body: &str,
        response_status: u16,
        tool_name: &str,
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
            "INSERT INTO detections (id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![id, timestamp_ms, rule_id, rule_name, strategy, placeholder, encrypted_original, encrypted_context, request_path, sanitized_body, response_status, tool_name],
        )?;

        tracing::debug!("detection recorded: {} -> {}", rule_id, placeholder);
        Ok(())
    }

    /// 批量记录检测结果，减少数据库锁竞争。
    /// 适用于高频写入场景，可显著提升吞吐量。
    pub fn batch_record(
        &self,
        records: &[DetectionRecord],
    ) -> Result<usize, anyhow::Error> {
        if records.is_empty() {
            return Ok(0);
        }

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let mut count = 0;
        for record in records {
            let encrypted_original = self.encrypt(record.original.as_bytes())?;
            let encrypted_context = self.encrypt(record.context.as_bytes())?;

            tx.execute(
                "INSERT INTO detections (id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12",
                rusqlite::params![
                    &record.id,
                    record.timestamp_ms,
                    &record.rule_id,
                    &record.rule_name,
                    &record.strategy,
                    &record.placeholder,
                    encrypted_original,
                    encrypted_context,
                    &record.request_path,
                    &record.sanitized_body,
                    record.response_status as i64,
                    &record.tool_name,
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        tracing::debug!("batch recorded {} detections", count);
        Ok(count)
    }

    /// 分页查询检测记录，按时间倒序，返回时解密 `original` 和 `context` 字段。
    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name
             FROM detections ORDER BY timestamp_ms DESC LIMIT ?1 OFFSET ?2",
        )?;

        let rows = stmt.query_map(rusqlite::params![limit as i64, offset as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get::<_, Vec<u8>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (id, timestamp_ms, rule_id, rule_name, strategy, placeholder, enc_original, enc_context, request_path, sanitized_body, response_status, tool_name) = row?;
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
                rule_name,
                strategy,
                placeholder,
                original,
                context,
                request_path,
                sanitized_body,
                response_status: response_status as u16,
                tool_name,
            });
        }

        Ok(records)
    }

    /// 检测记录总数
    pub fn count(&self) -> Result<usize, anyhow::Error> {
        self.count_filtered(None, None, None, None, None)
    }

    /// 按过滤条件统计记录数，参数含义与 `list_filtered` 一致。
    pub fn count_filtered(
        &self,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
        strategy_filter: Option<&str>,
    ) -> Result<usize, anyhow::Error> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from("SELECT COUNT(*) FROM detections WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(rule_id) = rule_id_filter {
            sql.push_str(" AND rule_id = ?");
            params.push(Box::new(rule_id.to_string()));
        }
        if let Some(path) = path_filter {
            sql.push_str(" AND request_path LIKE ?");
            params.push(Box::new(format!("%{}%", path)));
        }
        if let Some(from) = date_from_ms {
            sql.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(from));
        }
        if let Some(to) = date_to_ms {
            sql.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(to));
        }
        if let Some(strategy) = strategy_filter {
            sql.push_str(" AND strategy = ?");
            params.push(Box::new(strategy.to_string()));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count as usize)
    }

    /// 按条件过滤的分页查询。未提供的过滤条件不做筛选。
    pub fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
        strategy_filter: Option<&str>,
    ) -> Result<Vec<DetectionRecord>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            "SELECT id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name FROM detections WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(rule_id) = rule_id_filter {
            sql.push_str(" AND rule_id = ?");
            params.push(Box::new(rule_id.to_string()));
        }
        if let Some(path) = path_filter {
            sql.push_str(" AND request_path LIKE ?");
            params.push(Box::new(format!("%{}%", path)));
        }
        if let Some(from) = date_from_ms {
            sql.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(from));
        }
        if let Some(to) = date_to_ms {
            sql.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(to));
        }
        if let Some(strategy) = strategy_filter {
            sql.push_str(" AND strategy = ?");
            params.push(Box::new(strategy.to_string()));
        }

        sql.push_str(" ORDER BY timestamp_ms DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit as i64));
        params.push(Box::new(offset as i64));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get::<_, Vec<u8>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (id, timestamp_ms, rule_id, rule_name, strategy, placeholder, enc_original, enc_context, request_path, sanitized_body, response_status, tool_name) = row?;
            let original = self.decrypt_string(&enc_original);
            let context = self.decrypt_string(&enc_context);
            records.push(DetectionRecord {
                id,
                timestamp_ms,
                rule_id,
                rule_name,
                strategy,
                placeholder,
                original,
                context,
                request_path,
                sanitized_body,
                response_status: response_status as u16,
                tool_name,
            });
        }
        Ok(records)
    }

    /// 按 (rule_id, strategy) 分组查询，返回每组最新时间与计数，按最新时间降序。
    pub fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
    ) -> Result<Vec<AuditGroup>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            "SELECT rule_id, MAX(rule_name), strategy, COUNT(*) AS cnt, MAX(timestamp_ms) AS latest_ts \
             FROM detections WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(rule_id) = rule_id_filter {
            sql.push_str(" AND rule_id = ?");
            params.push(Box::new(rule_id.to_string()));
        }
        if let Some(path) = path_filter {
            sql.push_str(" AND request_path LIKE ?");
            params.push(Box::new(format!("%{}%", path)));
        }
        if let Some(from) = date_from_ms {
            sql.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(from));
        }
        if let Some(to) = date_to_ms {
            sql.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(to));
        }

        sql.push_str(" GROUP BY rule_id, strategy ORDER BY latest_ts DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit as i64));
        params.push(Box::new(offset as i64));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(AuditGroup {
                rule_id: row.get(0)?,
                rule_name: row.get::<_, String>(1).unwrap_or_default(),
                strategy: row.get(2)?,
                count: row.get::<_, i64>(3)? as usize,
                latest_timestamp_ms: row.get(4)?,
            })
        })?;

        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }

    /// 分组后的总组数（用于分页）
    pub fn count_grouped(
        &self,
        rule_id_filter: Option<&str>,
        path_filter: Option<&str>,
        date_from_ms: Option<i64>,
        date_to_ms: Option<i64>,
    ) -> Result<usize, anyhow::Error> {
        let conn = self.conn.lock().unwrap();

        let mut sql = String::from(
            "SELECT COUNT(*) FROM (SELECT 1 FROM detections WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(rule_id) = rule_id_filter {
            sql.push_str(" AND rule_id = ?");
            params.push(Box::new(rule_id.to_string()));
        }
        if let Some(path) = path_filter {
            sql.push_str(" AND request_path LIKE ?");
            params.push(Box::new(format!("%{}%", path)));
        }
        if let Some(from) = date_from_ms {
            sql.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(from));
        }
        if let Some(to) = date_to_ms {
            sql.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(to));
        }

        sql.push_str(" GROUP BY rule_id, strategy)");

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let count: i64 = conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))?;
        Ok(count as usize)
    }

    /// 按 ID 的单条查询。
    pub fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name FROM detections WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(rusqlite::params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Vec<u8>>(6)?,
                row.get::<_, Vec<u8>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })?;

        match rows.next() {
            Some(row) => {
                let (id, ts, rule_id, rule_name, strategy, placeholder, enc_orig, enc_ctx, path, body, status, tool_name) = row?;
                let original = self.decrypt_string(&enc_orig);
                let context = self.decrypt_string(&enc_ctx);
                Ok(Some(DetectionRecord {
                    id,
                    timestamp_ms: ts,
                    rule_id,
                    rule_name,
                    strategy,
                    placeholder,
                    original,
                    context,
                    request_path: path,
                    sanitized_body: body,
                    response_status: status as u16,
                    tool_name,
                }))
            }
            None => Ok(None),
        }
    }

    /// 删除单条记录。
    pub fn delete(&self, id: &str) -> Result<bool, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM detections WHERE id = ?1", rusqlite::params![id])?;
        Ok(affected > 0)
    }

    /// 获取最近 N 条检测记录（供仪表盘事件列表使用）。
    pub fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, anyhow::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp_ms, rule_id, rule_name, strategy, placeholder, original_encrypted, context_encrypted, request_path, sanitized_body, response_status, tool_name FROM detections ORDER BY timestamp_ms DESC LIMIT ?1"
        )?;
        let records: Vec<DetectionRecord> = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                    row.get::<_, Vec<u8>>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, String>(11)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, ts, rule_id, rule_name, strategy, placeholder, enc_orig, enc_ctx, path, body, status, tool_name)| {
                let original = self.decrypt_string(&enc_orig);
                let context = self.decrypt_string(&enc_ctx);
                DetectionRecord {
                    id,
                    timestamp_ms: ts,
                    rule_id,
                    rule_name,
                    strategy,
                    placeholder,
                    original,
                    context,
                    request_path: path,
                    sanitized_body: body,
                    response_status: status as u16,
                    tool_name,
                }
            })
            .collect();
        Ok(records)
    }

    /// 汇总统计数据。
    pub fn stats(&self) -> Result<AuditStats, anyhow::Error> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM detections", [], |row| row.get(0))?;

        // 今天零点 (UTC ms)
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let day_ms: i64 = 24 * 60 * 60 * 1000;
        let today_start = now_ms - (now_ms % day_ms);

        let today: i64 = conn.query_row(
            "SELECT COUNT(*) FROM detections WHERE timestamp_ms >= ?1",
            rusqlite::params![today_start],
            |row| row.get(0),
        )?;

        let week_start = today_start - 7 * day_ms;
        let week: i64 = conn.query_row(
            "SELECT COUNT(*) FROM detections WHERE timestamp_ms >= ?1",
            rusqlite::params![week_start],
            |row| row.get(0),
        )?;

        // 规则分布
        let mut stmt = conn.prepare(
            "SELECT rule_id, rule_name, COUNT(*) as cnt FROM detections GROUP BY rule_id ORDER BY cnt DESC",
        )?;
        let rule_dist: Vec<RuleCount> = stmt
            .query_map([], |row| {
                Ok(RuleCount {
                    rule_id: row.get(0)?,
                    rule_name: row.get::<_, String>(1).unwrap_or_default(),
                    count: row.get::<_, i64>(2)? as usize,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 数据库文件大小
        let db_size = std::fs::metadata(
            conn.query_row::<String, _, _>("PRAGMA database_list", [], |row| row.get(2))?
        )
        .map(|m| m.len())
        .unwrap_or(0);

        Ok(AuditStats {
            total_count: total as usize,
            today_count: today as usize,
            week_count: week as usize,
            rule_distribution: rule_dist,
            db_size_bytes: db_size,
        })
    }

    fn decrypt_string(&self, data: &[u8]) -> String {
        self.decrypt(data)
            .map(|bytes| String::from_utf8(bytes).unwrap_or_else(|_| "<非 UTF-8 数据>".to_string()))
            .unwrap_or_else(|_| "<解密失败>".to_string())
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
        let mut out = nonce.to_vec();
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
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

/// 使用 PBKDF2-HMAC-SHA256 从密码派生 AES-256 密钥。
/// Salt 存储在 `{db_path}.salt` 文件中，首次使用时自动生成。
fn derive_cipher(db_path: &Path, encryption_key: &str) -> Result<Aes256Gcm, anyhow::Error> {
    let salt_path = db_path.with_extension("db.salt");

    let salt: [u8; SALT_LEN] = if salt_path.exists() {
        let bytes = std::fs::read(&salt_path)
            .with_context(|| format!("无法读取 salt 文件: {}", salt_path.display()))?;
        if bytes.len() != SALT_LEN {
            return Err(anyhow::anyhow!(
                "salt 文件长度异常: 期望 {} 字节，实际 {} 字节",
                SALT_LEN, bytes.len()
            ));
        }
        let mut arr = [0u8; SALT_LEN];
        arr.copy_from_slice(&bytes);
        arr
    } else {
        use rand::RngCore;
        let mut arr = [0u8; SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut arr);
        if let Some(parent) = salt_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&salt_path, arr)?;
        arr
    };

    let key_bytes = pbkdf2::pbkdf2_hmac_array::<Sha256, 32>(
        encryption_key.as_bytes(),
        &salt,
        PBKDF2_ITERATIONS,
    );
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
    Ok(Aes256Gcm::new(key))
}

// ── AuditStorage Trait Implementation ──

impl AuditStorage for Storage {
    fn record(
        &self,
        rule_id: &str,
        rule_name: &str,
        strategy: &str,
        placeholder: &str,
        original: &str,
        context: &str,
        request_path: &str,
        sanitized_body: &str,
        response_status: u16,
        tool_name: &str,
    ) -> Result<(), StorageError> {
        Storage::record(self, rule_id, rule_name, strategy, placeholder, original, context, request_path, sanitized_body, response_status, tool_name)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError> {
        Storage::batch_record(self, records)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        Storage::list(self, limit, offset)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError> {
        Storage::list_filtered(
            self,
            limit,
            offset,
            filter.rule_id.as_deref(),
            filter.path.as_deref(),
            filter.date_from_ms,
            filter.date_to_ms,
            filter.strategy.as_deref(),
        )
        .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, StorageError> {
        Storage::get_by_id(self, id)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        Storage::list_recent(self, limit)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<AuditGroup>, StorageError> {
        Storage::list_grouped(
            self,
            limit,
            offset,
            filter.rule_id.as_deref(),
            filter.path.as_deref(),
            filter.date_from_ms,
            filter.date_to_ms,
        )
        .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn count_grouped(&self, filter: AuditFilter) -> Result<usize, StorageError> {
        Storage::count_grouped(
            self,
            filter.rule_id.as_deref(),
            filter.path.as_deref(),
            filter.date_from_ms,
            filter.date_to_ms,
        )
        .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn count(&self) -> Result<usize, StorageError> {
        Storage::count(self)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn count_filtered(&self, filter: AuditFilter) -> Result<usize, StorageError> {
        Storage::count_filtered(
            self,
            filter.rule_id.as_deref(),
            filter.path.as_deref(),
            filter.date_from_ms,
            filter.date_to_ms,
            filter.strategy.as_deref(),
        )
        .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn stats(&self) -> Result<AuditStats, StorageError> {
        Storage::stats(self)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn delete(&self, id: &str) -> Result<bool, StorageError> {
        Storage::delete(self, id)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }

    fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError> {
        Storage::purge_before(self, timestamp_ms)
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))
    }
}
