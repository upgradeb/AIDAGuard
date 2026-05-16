# DetectionEngine Trait 优化规划

**工作项：** 3.3  
**优先级：** P2  
**工作量：** 1-2 天  
**依赖：** 3.1 依赖重构

---

## 一、目标

规范化检测引擎接口，支持：
- 多种检测引擎实现
- 统一的调用方式
- 便于测试和扩展

---

## 二、当前状态

### 2.1 现有 Trait 定义

```rust
// aidaguard-core/src/engine.rs

pub trait DetectionEngine: Send + Sync {
    /// 检测文本中的敏感数据
    fn detect(&self, text: &str) -> Vec<Match>;
    
    /// 已加载规则数量
    fn rule_count(&self) -> usize;
    
    /// 根据 ID 查询规则名称
    fn rule_name(&self, id: &str) -> Option<&str>;
    
    /// 从目录重新加载规则
    fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error>;
    
    /// 从预设目录加载规则
    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, anyhow::Error>;
}
```

### 2.2 现有实现

| 实现者 | 位置 | 说明 |
|--------|------|------|
| `Detector` | `aidaguard-core/src/detector/mod.rs` | 基础正则检测器 |
| `AnalyzerEngine` | `aidaguard-detector/src/pipeline.rs` | 完整检测管线（正则 + NLP） |

### 2.3 问题分析

当前 trait 设计基本合理，但可以增强：

1. **缺少并行检测方法** - `AnalyzerEngine` 有 `scan_parallel()` 但 trait 未定义
2. **缺少元信息查询** - 如支持的实体类型
3. **错误类型不统一** - 使用 `anyhow::Error` 而非 `DetectionError`

---

## 三、优化方案

### 3.1 增强 Trait 定义

```rust
// aidaguard-core/src/engine.rs

use crate::entity::EntityType;
use crate::error::DetectionError;
use crate::detector::Match;
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
    fn rule_ids(&self) -> Vec<String>;
    
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
    fn supported_entities(&self) -> Vec<EntityType>;
    
    /// 是否支持指定实体类型
    fn supports(&self, entity_type: &EntityType) -> bool {
        self.supported_entities().contains(entity_type)
    }
    
    /// 引擎名称
    fn name(&self) -> &str;
    
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
```

### 3.2 更新 Detector 实现

```rust
// aidaguard-core/src/detector/mod.rs

impl DetectionEngine for Detector {
    fn detect(&self, text: &str) -> Vec<Match> {
        self.detect(text)
    }
    
    fn detect_parallel(&self, text: &str) -> Vec<Match> {
        // Detector 本身不并行，直接调用 detect
        self.detect(text)
    }
    
    fn rule_count(&self) -> usize {
        self.rules.len()
    }
    
    fn rule_name(&self, id: &str) -> Option<&str> {
        self.rules.iter()
            .find(|r| r.def.id == id)
            .map(|r| r.def.name.as_str())
    }
    
    fn rule_ids(&self) -> Vec<String> {
        self.rules.iter()
            .map(|r| r.def.id.clone())
            .collect()
    }
    
    fn reload(&mut self, dir: &Path) -> Result<usize, DetectionError> {
        self.load_from_dir(dir)
            .map_err(|e| DetectionError::RuleCompilation(e.to_string()))
    }
    
    fn supported_entities(&self) -> Vec<EntityType> {
        // 从规则推断支持的实体类型
        self.rules.iter()
            .filter_map(|r| EntityType::from_str(&r.def.id).ok())
            .collect()
    }
    
    fn name(&self) -> &str {
        "Detector"
    }
}
```

### 3.3 更新 AnalyzerEngine 实现

```rust
// aidaguard-detector/src/pipeline.rs

impl DetectionEngine for AnalyzerEngine {
    fn detect(&self, text: &str) -> Vec<Match> {
        self.analyze(text)
    }
    
    fn detect_parallel(&self, text: &str) -> Vec<Match> {
        // 使用 rayon 并行检测
        self.analyze_all_parallel(text)
    }
    
    fn rule_count(&self) -> usize {
        self.registry.recognizer_count()
    }
    
    fn rule_name(&self, id: &str) -> Option<&str> {
        self.registry.get_recognizer(id)
            .map(|r| r.name())
    }
    
    fn rule_ids(&self) -> Vec<String> {
        self.registry.recognizer_ids()
    }
    
    fn reload(&mut self, dir: &Path) -> Result<usize, DetectionError> {
        self.reload_rules(dir)
            .map_err(|e| DetectionError::RuleCompilation(e.to_string()))
    }
    
    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, DetectionError> {
        self.reload_from_presets(base_dir, presets)
            .map_err(|e| DetectionError::RuleCompilation(e.to_string()))
    }
    
    fn supported_entities(&self) -> Vec<EntityType> {
        self.registry.get_supported_entities()
    }
    
    fn name(&self) -> &str {
        "AnalyzerEngine"
    }
}
```

---

## 四、新增功能

### 4.1 检测上下文

```rust
// aidaguard-core/src/engine.rs

/// 检测上下文
///
/// 提供检测时的额外信息，用于上下文感知检测。
#[derive(Debug, Clone, Default)]
pub struct DetectionContext {
    /// 请求路径
    pub request_path: Option<String>,
    
    /// 工具名称
    pub tool_name: Option<String>,
    
    /// 用户 ID
    pub user_id: Option<String>,
    
    /// 自定义元数据
    pub metadata: HashMap<String, String>,
}

impl DetectionEngine for dyn DetectionEngine {
    /// 带上下文的检测
    ///
    /// 默认实现忽略上下文，实现者可覆盖以利用上下文信息。
    fn detect_with_context(
        &self,
        text: &str,
        context: &DetectionContext,
    ) -> Vec<Match> {
        // 默认：忽略上下文
        let _ = context;
        self.detect(text)
    }
}
```

### 4.2 批量检测

```rust
impl DetectionEngine for dyn DetectionEngine {
    /// 批量检测多个文本
    ///
    /// 默认实现顺序处理，实现者可覆盖以并行处理。
    fn detect_batch(&self, texts: &[&str]) -> Vec<Vec<Match>> {
        texts.iter().map(|t| self.detect(t)).collect()
    }
    
    /// 并行批量检测
    fn detect_batch_parallel(&self, texts: &[&str]) -> Vec<Vec<Match>> {
        use rayon::prelude::*;
        texts.par_iter().map(|t| self.detect_parallel(t)).collect()
    }
}
```

---

## 五、使用示例

### 5.1 基本使用

```rust
use aidaguard_core::engine::DetectionEngine;
use aidaguard_detector::AnalyzerEngine;

// 创建引擎
let engine = AnalyzerEngine::builder()
    .with_rules_dir(Path::new("./rules"))
    .build()?;

// 检测
let matches = engine.detect("我的身份证号是 310101199001011234");

// 并行检测（更快）
let matches = engine.detect_parallel(&long_text);

// 查询能力
println!("引擎: {}", engine.name());
println!("规则数: {}", engine.rule_count());
println!("支持: {:?}", engine.supported_entities());
```

### 5.2 通过 Trait 使用

```rust
fn process_with_engine(engine: &dyn DetectionEngine, text: &str) {
    let matches = engine.detect(text);
    
    for m in &matches {
        println!("发现: {} -> {}", m.rule_id, m.text);
    }
}

// 可以传入任何实现
process_with_engine(&detector, text);
process_with_engine(&analyzer, text);
```

### 5.3 测试中使用 Mock

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    /// Mock 检测引擎
    struct MockEngine {
        matches: Vec<Match>,
    }
    
    impl DetectionEngine for MockEngine {
        fn detect(&self, _text: &str) -> Vec<Match> {
            self.matches.clone()
        }
        
        fn rule_count(&self) -> usize { 1 }
        fn rule_name(&self, _id: &str) -> Option<&str> { Some("mock") }
        fn rule_ids(&self) -> Vec<String> { vec!["mock".into()] }
        fn reload(&mut self, _dir: &Path) -> Result<usize, DetectionError> { Ok(1) }
        fn supported_entities(&self) -> Vec<EntityType> { vec![] }
        fn name(&self) -> &str { "MockEngine" }
    }
    
    #[test]
    fn test_with_mock() {
        let mock = MockEngine {
            matches: vec![Match {
                rule_id: "test".into(),
                text: "敏感数据".into(),
                // ...
            }],
        };
        
        // 使用 mock 测试
        let result = mock.detect("任意文本");
        assert_eq!(result.len(), 1);
    }
}
```

---

## 六、文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `aidaguard-core/src/engine.rs` | 修改 | 增强 trait 定义 |
| `aidaguard-core/src/detector/mod.rs` | 修改 | 更新实现 |
| `aidaguard-detector/src/pipeline.rs` | 修改 | 更新实现 |

---

## 七、验收标准

- [ ] Trait 定义完整，包含所有方法
- [ ] `Detector` 实现更新
- [ ] `AnalyzerEngine` 实现更新
- [ ] 新增方法有默认实现
- [ ] 单元测试通过
- [ ] 文档注释完整
