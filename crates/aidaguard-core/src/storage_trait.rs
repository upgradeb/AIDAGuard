//! Audit storage interface trait.
//!
//! Defines the abstract interface for audit storage backends.
//! Implementations can be SQLite, in-memory, PostgreSQL, etc.

use crate::error::StorageError;
use crate::storage_types::{AuditFilter, AuditGroup, AuditStats, DetectionRecord};

/// 审计存储接口
///
/// 所有存储后端必须实现此 trait。
/// 提供检测记录的存储、查询、统计功能。
pub trait AuditStorage: Send + Sync {
    // ── 写入操作 ──

    /// 记录单条检测结果
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
    ) -> Result<(), StorageError>;

    /// 批量记录检测结果（高性能写入）
    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError>;

    // ── 查询操作 ──

    /// 分页查询记录（按时间倒序）
    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError>;

    /// 条件查询记录
    fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError>;

    /// 按 ID 查询单条记录
    fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, StorageError>;

    /// 查询最近记录
    fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, StorageError>;

    // ── 分组查询 ──

    /// 按规则和策略分组查询
    fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<AuditGroup>, StorageError>;

    /// 分组计数
    fn count_grouped(&self, filter: AuditFilter) -> Result<usize, StorageError>;

    // ── 统计操作 ──

    /// 总记录数
    fn count(&self) -> Result<usize, StorageError>;

    /// 条件计数
    fn count_filtered(&self, filter: AuditFilter) -> Result<usize, StorageError>;

    /// 统计信息
    fn stats(&self) -> Result<AuditStats, StorageError>;

    // ── 删除操作 ──

    /// 删除单条记录
    fn delete(&self, id: &str) -> Result<bool, StorageError>;

    /// 清理过期记录
    fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError>;
}
