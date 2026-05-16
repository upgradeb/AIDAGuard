//! Detection engine interface trait.
//!
//! Defines the abstract interface for sensitive data detection engines.
//! Implementations can be basic regex detectors or full NLP pipelines.

use crate::detector::Match;
use crate::entity::EntityType;
use crate::error::DetectionError;
use std::path::Path;

/// 检测引擎接口
///
/// 定义敏感数据检测的抽象接口。
/// 实现可以是：
/// - 基础正则检测器 (`Detector`)
/// - 完整检测管线 (`AnalyzerEngine`)
/// - 自定义检测器
pub trait DetectionEngine: Send + Sync {
    // ── 核心检测 ──

    /// 检测文本中的敏感数据
    ///
    /// 返回所有匹配项，按优先级和位置排序。
    fn detect(&self, text: &str) -> Vec<Match>;

    /// 并行检测（性能优化）
    ///
    /// 默认实现调用 `detect()`，实现者可覆盖以提供并行版本。
    fn detect_parallel(&self, text: &str) -> Vec<Match> {
        self.detect(text)
    }

    // ── 规则管理 ──

    /// 已加载规则数量
    fn rule_count(&self) -> usize;

    /// 根据 ID 查询规则名称
    fn rule_name(&self, id: &str) -> Option<&str>;

    /// 获取所有规则 ID
    fn rule_ids(&self) -> Vec<String> {
        Vec::new() // 默认空实现
    }

    /// 从目录重新加载规则
    fn reload(&mut self, dir: &Path) -> Result<usize, DetectionError>;

    /// 从预设目录加载规则
    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, DetectionError> {
        // 默认实现：直接加载 base_dir
        self.reload(base_dir)
    }

    // ── 能力查询 ──

    /// 支持的实体类型
    ///
    /// 返回此引擎能检测的所有实体类型。
    fn supported_entities(&self) -> Vec<EntityType> {
        Vec::new() // 默认空实现
    }

    /// 是否支持指定实体类型
    fn supports(&self, entity_type: &EntityType) -> bool {
        self.supported_entities().contains(entity_type)
    }

    /// 引擎名称
    fn name(&self) -> &str {
        "DetectionEngine"
    }

    /// 引擎版本
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    // ── 统计信息 ──

    /// 检测统计
    fn stats(&self) -> EngineStats {
        EngineStats {
            name: self.name().to_string(),
            rule_count: self.rule_count(),
            supported_entities: self.supported_entities().len(),
        }
    }
}

/// 引擎统计信息
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub name: String,
    pub rule_count: usize,
    pub supported_entities: usize,
}
