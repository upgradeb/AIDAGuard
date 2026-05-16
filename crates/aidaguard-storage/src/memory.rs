//! In-memory storage implementation for testing and temporary use.
//!
//! No persistence, all data is lost when the process exits.
//! Useful for unit tests and scenarios where persistence is not needed.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use aidaguard_core::error::StorageError;
use aidaguard_core::storage_trait::AuditStorage;
use aidaguard_core::storage_types::{
    AuditFilter, AuditGroup, AuditStats, DetectionRecord, RuleCount,
};

/// 内存存储实现
///
/// 所有数据存储在内存中，进程退出后丢失。
/// 适用于测试和不需要持久化的场景。
pub struct MemoryStorage {
    records: Arc<RwLock<Vec<DetectionRecord>>>,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditStorage for MemoryStorage {
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
        let record = DetectionRecord {
            id: Uuid::new_v4().to_string(),
            timestamp_ms: current_timestamp_ms(),
            rule_id: rule_id.to_string(),
            rule_name: rule_name.to_string(),
            strategy: strategy.to_string(),
            placeholder: placeholder.to_string(),
            original: original.to_string(),
            context: context.to_string(),
            request_path: request_path.to_string(),
            sanitized_body: sanitized_body.to_string(),
            response_status,
            tool_name: tool_name.to_string(),
        };

        let mut records = self.records.blocking_write();
        records.push(record);
        Ok(())
    }

    fn batch_record(&self, records: &[DetectionRecord]) -> Result<usize, StorageError> {
        let mut storage = self.records.blocking_write();
        storage.extend(records.iter().cloned());
        Ok(records.len())
    }

    fn list(&self, limit: usize, offset: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        let records = self.records.blocking_read();
        Ok(records
            .iter()
            .rev() // 按时间倒序
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    fn list_filtered(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<DetectionRecord>, StorageError> {
        let records = self.records.blocking_read();
        Ok(records
            .iter()
            .rev()
            .filter(|r| Self::matches_filter(r, &filter))
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    fn get_by_id(&self, id: &str) -> Result<Option<DetectionRecord>, StorageError> {
        let records = self.records.blocking_read();
        Ok(records.iter().find(|r| r.id == id).cloned())
    }

    fn list_recent(&self, limit: usize) -> Result<Vec<DetectionRecord>, StorageError> {
        self.list(limit, 0)
    }

    fn list_grouped(
        &self,
        limit: usize,
        offset: usize,
        filter: AuditFilter,
    ) -> Result<Vec<AuditGroup>, StorageError> {
        let records = self.records.blocking_read();

        // 按 (rule_id, strategy) 分组
        let mut groups: HashMap<(String, String), Vec<&DetectionRecord>> = HashMap::new();
        for record in records.iter() {
            if Self::matches_filter(record, &filter) {
                let key = (record.rule_id.clone(), record.strategy.clone());
                groups.entry(key).or_default().push(record);
            }
        }

        // 转换为 AuditGroup 并按最新时间排序
        let mut result: Vec<AuditGroup> = groups
            .into_iter()
            .map(|((rule_id, strategy), recs)| {
                let latest = recs.iter().map(|r| r.timestamp_ms).max().unwrap_or(0);
                let rule_name = recs
                    .first()
                    .map(|r| r.rule_name.as_str())
                    .unwrap_or("");
                AuditGroup {
                    rule_id,
                    rule_name: rule_name.to_string(),
                    strategy,
                    count: recs.len(),
                    latest_timestamp_ms: latest,
                }
            })
            .collect();

        result.sort_by(|a, b| b.latest_timestamp_ms.cmp(&a.latest_timestamp_ms));

        Ok(result.into_iter().skip(offset).take(limit).collect())
    }

    fn count_grouped(&self, filter: AuditFilter) -> Result<usize, StorageError> {
        let records = self.records.blocking_read();
        let mut groups: HashMap<(String, String), ()> = HashMap::new();
        for record in records.iter() {
            if Self::matches_filter(record, &filter) {
                let key = (record.rule_id.clone(), record.strategy.clone());
                groups.entry(key).or_insert(());
            }
        }
        Ok(groups.len())
    }

    fn count(&self) -> Result<usize, StorageError> {
        Ok(self.records.blocking_read().len())
    }

    fn count_filtered(&self, filter: AuditFilter) -> Result<usize, StorageError> {
        let records = self.records.blocking_read();
        Ok(records.iter().filter(|r| Self::matches_filter(r, &filter)).count())
    }

    fn stats(&self) -> Result<AuditStats, StorageError> {
        let records = self.records.blocking_read();

        let now = current_timestamp_ms();
        let day_ms = 24 * 60 * 60 * 1000;
        let today_start = now - (now % day_ms);
        let week_start = today_start - 7 * day_ms;

        // 规则分布
        let mut rule_counts: HashMap<String, usize> = HashMap::new();
        for record in records.iter() {
            *rule_counts.entry(record.rule_id.clone()).or_insert(0) += 1;
        }
        let rule_distribution: Vec<RuleCount> = records
            .iter()
            .map(|r| (r.rule_id.clone(), r.rule_name.clone()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|(rule_id, rule_name)| RuleCount {
                rule_id: rule_id.clone(),
                rule_name,
                count: *rule_counts.get(&rule_id).unwrap_or(&0),
            })
            .collect();

        Ok(AuditStats {
            total_count: records.len(),
            today_count: records
                .iter()
                .filter(|r| r.timestamp_ms >= today_start)
                .count(),
            week_count: records
                .iter()
                .filter(|r| r.timestamp_ms >= week_start)
                .count(),
            rule_distribution,
            db_size_bytes: 0, // 内存存储无文件大小
        })
    }

    fn delete(&self, id: &str) -> Result<bool, StorageError> {
        let mut records = self.records.blocking_write();
        let before = records.len();
        records.retain(|r| r.id != id);
        Ok(records.len() < before)
    }

    fn purge_before(&self, timestamp_ms: i64) -> Result<usize, StorageError> {
        let mut records = self.records.blocking_write();
        let before = records.len();
        records.retain(|r| r.timestamp_ms >= timestamp_ms);
        Ok(before - records.len())
    }
}

impl MemoryStorage {
    /// 检查记录是否匹配过滤器
    fn matches_filter(record: &DetectionRecord, filter: &AuditFilter) -> bool {
        if let Some(ref rule_id) = filter.rule_id {
            if record.rule_id != *rule_id {
                return false;
            }
        }
        if let Some(ref path) = filter.path {
            if !record.request_path.contains(path) {
                return false;
            }
        }
        if let Some(from) = filter.date_from_ms {
            if record.timestamp_ms < from {
                return false;
            }
        }
        if let Some(to) = filter.date_to_ms {
            if record.timestamp_ms > to {
                return false;
            }
        }
        if let Some(ref strategy) = filter.strategy {
            if record.strategy != *strategy {
                return false;
            }
        }
        if let Some(ref tool) = filter.tool_name {
            if record.tool_name != *tool {
                return false;
            }
        }
        true
    }
}

/// 获取当前时间戳（毫秒）
fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_basic() {
        let storage = MemoryStorage::new();

        // 记录
        storage
            .record(
                "id_card",
                "身份证号",
                "mask",
                "[[ID]]",
                "310101199001011234",
                "用户输入",
                "/api/chat",
                "",
                200,
                "cursor",
            )
            .unwrap();

        // 查询
        let records = storage.list(10, 0).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].rule_id, "id_card");
    }

    #[test]
    fn test_memory_storage_filter() {
        let storage = MemoryStorage::new();

        storage.record("rule1", "规则1", "mask", "", "", "", "/api/a", "", 200, "").unwrap();
        storage.record("rule2", "规则2", "mask", "", "", "", "/api/b", "", 200, "").unwrap();

        let filter = AuditFilter::by_rule("rule1");
        let records = storage.list_filtered(10, 0, filter).unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_memory_storage_stats() {
        let storage = MemoryStorage::new();

        storage.record("rule1", "规则1", "mask", "", "", "", "", "", 200, "").unwrap();
        storage.record("rule1", "规则1", "mask", "", "", "", "", "", 200, "").unwrap();
        storage.record("rule2", "规则2", "mask", "", "", "", "", "", 200, "").unwrap();

        let stats = storage.stats().unwrap();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.rule_distribution.len(), 2);
    }
}
