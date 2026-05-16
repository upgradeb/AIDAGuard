//! Storage-related types shared across crates.
//!
//! These types are defined in `aidaguard-core` so that both
//! `aidaguard-storage` (implementation) and other crates can
//! reference them without creating a circular dependency.

use serde::{Deserialize, Serialize};

/// 一条检测审计记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionRecord {
    pub id: String,
    pub timestamp_ms: i64,
    pub rule_id: String,
    pub rule_name: String,
    pub strategy: String,
    pub placeholder: String,
    pub original: String,
    pub context: String,
    pub request_path: String,
    pub sanitized_body: String,
    pub response_status: u16,
    pub tool_name: String,
}

impl Default for DetectionRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            timestamp_ms: 0,
            rule_id: String::new(),
            rule_name: String::new(),
            strategy: String::new(),
            placeholder: String::new(),
            original: String::new(),
            context: String::new(),
            request_path: String::new(),
            sanitized_body: String::new(),
            response_status: 0,
            tool_name: String::new(),
        }
    }
}

/// 规则命中统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCount {
    pub rule_id: String,
    pub rule_name: String,
    pub count: usize,
}

/// 按 (rule_id, strategy) 分组的审计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditGroup {
    pub rule_id: String,
    pub rule_name: String,
    pub strategy: String,
    pub count: usize,
    pub latest_timestamp_ms: i64,
}

/// 审计汇总统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditStats {
    pub total_count: usize,
    pub today_count: usize,
    pub week_count: usize,
    pub rule_distribution: Vec<RuleCount>,
    pub db_size_bytes: u64,
}

/// 查询过滤器
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub rule_id: Option<String>,
    pub path: Option<String>,
    pub date_from_ms: Option<i64>,
    pub date_to_ms: Option<i64>,
    pub strategy: Option<String>,
    pub tool_name: Option<String>,
}

impl AuditFilter {
    /// 创建空过滤器
    pub fn new() -> Self {
        Self::default()
    }

    /// 按规则 ID 过滤
    pub fn by_rule(rule_id: impl Into<String>) -> Self {
        Self {
            rule_id: Some(rule_id.into()),
            ..Default::default()
        }
    }

    /// 按时间范围过滤
    pub fn by_date_range(from_ms: i64, to_ms: i64) -> Self {
        Self {
            date_from_ms: Some(from_ms),
            date_to_ms: Some(to_ms),
            ..Default::default()
        }
    }

    /// 链式设置规则 ID
    pub fn with_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.rule_id = Some(rule_id.into());
        self
    }

    /// 链式设置路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 链式设置时间范围
    pub fn with_date_range(mut self, from_ms: i64, to_ms: i64) -> Self {
        self.date_from_ms = Some(from_ms);
        self.date_to_ms = Some(to_ms);
        self
    }

    /// 链式设置策略
    pub fn with_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.strategy = Some(strategy.into());
        self
    }

    /// 链式设置工具名称
    pub fn with_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }
}
