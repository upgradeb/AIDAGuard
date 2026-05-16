//! Storage factory for creating storage backends.

use std::path::PathBuf;

use aidaguard_core::error::StorageError;
use aidaguard_core::storage_trait::AuditStorage;

use crate::memory::MemoryStorage;
use crate::Storage;

/// 存储工厂
///
/// 根据配置创建不同的存储后端实例。
pub struct StorageFactory;

impl StorageFactory {
    /// 根据配置创建存储实例
    ///
    /// # 参数
    /// - `storage_type`: 存储类型 ("sqlite" | "memory")
    /// - `db_path`: SQLite 数据库路径（仅 sqlite 需要）
    /// - `encryption_key`: 加密密钥（仅 sqlite 需要）
    ///
    /// # 示例
    /// ```ignore
    /// let storage = StorageFactory::create("sqlite", &path, Some("key"))?;
    /// let storage = StorageFactory::create("memory", &PathBuf::new(), None)?;
    /// ```
    pub fn create(
        storage_type: &str,
        db_path: &PathBuf,
        encryption_key: Option<&str>,
    ) -> Result<Box<dyn AuditStorage>, StorageError> {
        match storage_type {
            "sqlite" => {
                let key = encryption_key.unwrap_or_default();
                let storage = Storage::open(db_path, key)
                    .map_err(|e| StorageError::ConnectionFailed {
                        path: db_path.display().to_string(),
                        reason: e.to_string(),
                    })?;
                Ok(Box::new(storage))
            }
            "memory" => {
                let storage = MemoryStorage::new();
                Ok(Box::new(storage))
            }
            _ => Err(StorageError::UnknownType {
                type_name: storage_type.to_string(),
            }),
        }
    }

    /// 创建默认的 SQLite 存储
    ///
    /// 使用默认路径和密钥。
    pub fn default_sqlite() -> Result<Box<dyn AuditStorage>, StorageError> {
        let db_path = default_db_path();
        let storage = Storage::open(&db_path, "")
            .map_err(|e| StorageError::ConnectionFailed {
                path: db_path.display().to_string(),
                reason: e.to_string(),
            })?;
        Ok(Box::new(storage))
    }

    /// 创建内存存储（测试用）
    pub fn in_memory() -> Box<dyn AuditStorage> {
        Box::new(MemoryStorage::new())
    }
}

/// 获取默认数据库路径
fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aidaguard")
        .join("audit.db")
}
