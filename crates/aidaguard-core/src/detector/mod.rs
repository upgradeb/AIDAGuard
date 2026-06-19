//! Detector module with rule loading, compilation, and version management.

mod versioned;

use anyhow::{Context, Result};
use notify::{EventKind, RecursiveMode, Watcher};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// 安全编译正则，设置 size_limit 防止 ReDoS 攻击。
pub fn compile_regex(pattern: &str) -> Result<Regex> {
    // 限制模式长度，防止极端情况
    if pattern.len() > 2000 {
        return Err(anyhow::anyhow!("正则模式过长 ({} bytes)，上限 2000", pattern.len()));
    }
    RegexBuilder::new(pattern)
        .size_limit(1 << 20) // 1 MB DFA 大小限制
        .build()
        .with_context(|| format!("正则编译失败: {}", pattern))
}

/// 递归收集目录下所有 YAML 文件中的规则定义。
fn collect_yaml_files(dir: &Path, rules: &mut Vec<RuleDef>, count: &mut usize) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("无法读取规则目录: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, rules, count)?;
        } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            *count += 1;
            info!("加载规则文件: {}", path.display());
            let contents = std::fs::read_to_string(&path)
                .with_context(|| format!("无法读取文件: {}", path.display()))?;
            let rule_file: RuleFile = serde_yaml::from_str(&contents)
                .with_context(|| format!("YAML 解析失败: {}", path.display()))?;
            rules.extend(rule_file.rules);
        }
    }
    Ok(())
}

/// 替换策略
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// 整体替换为占位符，如 [[PHONE_a1b2c3d4]]
    Placeholder,
    /// 部分掩码，如 138****5678
    Mask,
}

/// 规则模式：仅检测 或 检测+过滤替换
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// 仅检测，记录但不替换
    Detect,
    /// 检测并过滤替换
    Filter,
}

fn default_enabled() -> bool {
    true
}
fn default_strategy() -> Strategy {
    Strategy::Placeholder
}
fn default_mode() -> Mode {
    Mode::Filter
}
fn default_priority() -> u32 {
    100
}
fn default_source() -> String {
    "system".to_string()
}

/// YAML 中单条规则的定义（未编译）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleDef {
    pub id: String,
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub exclude: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_strategy")]
    pub strategy: Strategy,
    #[serde(default = "default_mode")]
    pub mode: Mode,
    #[serde(default = "default_priority")]
    pub priority: u32,
    /// Applicable compliance frameworks (e.g. GDPR, PIPL, HIPAA, PCI_DSS)
    #[serde(default)]
    pub compliance: Vec<String>,
    /// Validator name for checksum validation (e.g. "luhn", "id_card_cn", "iban")
    #[serde(default)]
    pub validator: Option<String>,
    /// Context words that increase detection confidence
    #[serde(default)]
    pub context_words: Vec<String>,
    /// Base confidence score (0.0 - 1.0), used as starting point for context enhancement
    #[serde(default)]
    pub base_confidence: Option<f64>,
    /// Region code (e.g. "cn", "us", "eu", "sg", "jp", "gb", "kr", "global")
    #[serde(default)]
    pub region: Option<String>,
    /// Source of the rule: "system" (bundled) or "custom" (user-defined)
    #[serde(default = "default_source")]
    pub source: String,
}

/// YAML 规则文件顶层结构
#[derive(Debug, Deserialize, Serialize)]
pub struct RuleFile {
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub rules: Vec<RuleDef>,
}

/// Validator function type for checksum validation
pub type ValidatorFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// 编译后的规则
pub struct CompiledRule {
    pub def: RuleDef,
    pub regex: Regex,
    pub exclude_regex: Option<Regex>,
    /// Optional validator function for checksum validation (e.g., Luhn, mod-11)
    pub validator_fn: Option<ValidatorFn>,
}

impl std::fmt::Debug for CompiledRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledRule")
            .field("def", &self.def)
            .field("regex", &self.regex)
            .field("exclude_regex", &self.exclude_regex)
            .field("validator_fn", &self.validator_fn.as_ref().map(|_| "validator_fn"))
            .finish()
    }
}

impl Clone for CompiledRule {
    fn clone(&self) -> Self {
        Self {
            def: self.def.clone(),
            regex: self.regex.clone(),
            exclude_regex: self.exclude_regex.clone(),
            validator_fn: self.validator_fn.clone(),
        }
    }
}

/// 检测命中
#[derive(Debug, Clone)]
pub struct Match {
    pub rule_id: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub priority: u32,
    pub strategy: Strategy,
    pub mode: Mode,
    /// Confidence score (0.0 - 1.0), derived from base_confidence and context enhancement
    pub confidence: Option<f64>,
}

/// 敏感数据检测器
pub struct Detector {
    rules: Vec<CompiledRule>,
}

impl Detector {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 从指定目录递归加载所有 YAML 规则文件，替换当前规则集。
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let mut all_rules = Vec::new();
        let mut yaml_count = 0;
        collect_yaml_files(dir, &mut all_rules, &mut yaml_count)?;

        let count_before = self.rules.len();
        self.compile(all_rules);
        info!(
            "已从 {} 个 YAML 文件加载 {} 条规则（替换了原有的 {} 条）",
            yaml_count,
            self.rules.len(),
            count_before
        );
        Ok(self.rules.len())
    }

    /// 从指定目录加载所有 YAML 规则文件，追加到当前规则集（不替换）。
    pub fn append_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let mut all_rules = Vec::new();
        let mut yaml_count = 0;
        collect_yaml_files(dir, &mut all_rules, &mut yaml_count)?;
        self.compile_append(all_rules);
        Ok(self.rules.len())
    }

    /// 从基准目录加载多个规则预设。
    ///
    /// 支持两种结构：
    /// - **扁平结构**：预设名对应 `base_dir/{preset}.yaml` 文件
    /// - **目录结构**：预设名对应 `base_dir/{preset}/` 子目录
    ///
    /// 如果两者都存在，优先加载目录。
    pub fn load_from_presets(&mut self, base_dir: &Path, presets: &[&str]) -> Result<usize> {
        self.rules.clear();
        for preset in presets {
            let dir = base_dir.join(preset);
            let file = base_dir.join(format!("{}.yaml", preset));
            if dir.is_dir() {
                self.append_from_dir(&dir)?;
            } else if file.is_file() {
                self.append_from_file(&file)?;
            } else {
                warn!("规则预设不存在，跳过: {} (目录或文件)", preset);
            }
        }
        info!(
            "已从 {} 个预设加载 {} 条规则",
            presets.len(),
            self.rules.len()
        );
        Ok(self.rules.len())
    }

    /// 从单个 YAML 文件加载规则并追加到当前规则集。
    fn append_from_file(&mut self, path: &Path) -> Result<()> {
        info!("加载规则文件: {}", path.display());
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取文件: {}", path.display()))?;
        let rule_file: RuleFile = serde_yaml::from_str(&contents)
            .with_context(|| format!("YAML 解析失败: {}", path.display()))?;
        self.compile_append(rule_file.rules);
        Ok(())
    }

    /// 追加单条规则。如果 `def.enabled` 为 false，跳过编译并返回 Ok.
    pub fn add_rule(&mut self, def: RuleDef) -> Result<()> {
        if !def.enabled {
            return Ok(());
        }
        let regex = compile_regex(&def.pattern)
            .with_context(|| format!("规则 [{}] 的正则编译失败: {}", def.id, def.pattern))?;
        let exclude_regex = if let Some(ref excl) = def.exclude {
            Some(compile_regex(excl)
                .with_context(|| format!("规则 [{}] 的排除正则编译失败: {}", def.id, excl))?)
        } else {
            None
        };
        self.rules.push(CompiledRule {
            def,
            regex,
            exclude_regex,
            validator_fn: None, // Will be set by ValidatorRegistry
        });
        Ok(())
    }

    fn compile(&mut self, defs: Vec<RuleDef>) {
        let mut compiled = Vec::new();
        for def in defs {
            if !def.enabled {
                continue;
            }
            match compile_regex(&def.pattern) {
                Ok(regex) => {
                    let exclude_regex = def.exclude.as_ref().and_then(|excl| {
                        match compile_regex(excl) {
                            Ok(re) => Some(re),
                            Err(e) => {
                                warn!("规则 [{}] 的排除正则编译失败: {}", def.id, e);
                                None
                            }
                        }
                    });
                    compiled.push(CompiledRule {
                        def,
                        regex,
                        exclude_regex,
                        validator_fn: None, // Will be set by ValidatorRegistry
                    });
                }
                Err(e) => warn!("规则 [{}] 正则编译失败: {}", def.id, e),
            }
        }
        // 按优先级降序排列，高优先级先处理
        compiled.sort_by(|a, b| b.def.priority.cmp(&a.def.priority));
        self.rules = compiled;
    }

    fn compile_append(&mut self, defs: Vec<RuleDef>) {
        for def in defs {
            if !def.enabled {
                continue;
            }
            match compile_regex(&def.pattern) {
                Ok(regex) => {
                    let exclude_regex = def.exclude.as_ref().and_then(|excl| {
                        match compile_regex(excl) {
                            Ok(re) => Some(re),
                            Err(e) => {
                                warn!("规则 [{}] 的排除正则编译失败: {}", def.id, e);
                                None
                            }
                        }
                    });
                    self.rules.push(CompiledRule {
                        def,
                        regex,
                        exclude_regex,
                        validator_fn: None, // Will be set by ValidatorRegistry
                    });
                }
                Err(e) => warn!("规则 [{}] 正则编译失败: {}", def.id, e),
            }
        }
        // Re-sort after append: priority descending
        self.rules.sort_by(|a, b| b.def.priority.cmp(&a.def.priority));
    }

    /// 已编译的规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// 根据规则 ID 查询规则名称
    pub fn rule_name(&self, id: &str) -> Option<&str> {
        self.rules.iter().find(|r| r.def.id == id).map(|r| r.def.name.as_str())
    }

    /// 获取所有规则 ID
    pub fn rule_ids(&self) -> Vec<String> {
        self.rules.iter().map(|r| r.def.id.clone()).collect()
    }

    /// 获取所有需要校验器但尚未设置的规则 ID 和校验器名称
    pub fn rules_needing_validators(&self) -> Vec<(String, String)> {
        self.rules
            .iter()
            .filter(|r| r.validator_fn.is_none() && r.def.validator.is_some())
            .filter_map(|r| {
                r.def.validator.as_ref().map(|v| (r.def.id.clone(), v.clone()))
            })
            .collect()
    }

    /// 设置指定规则的校验器函数
    pub fn set_validator(&mut self, rule_id: &str, validator_fn: ValidatorFn) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.def.id == rule_id) {
            rule.validator_fn = Some(validator_fn);
            true
        } else {
            false
        }
    }

    /// 获取所有编译后的规则（只读引用）
    pub fn rules(&self) -> &[CompiledRule] {
        &self.rules
    }

    /// 在文本中检测敏感数据，返回去重且无重叠的命中列表
    pub fn detect(&self, text: &str) -> Vec<Match> {
        if self.rules.is_empty() {
            return Vec::new();
        }

        // 收集所有命中
        let mut matches: Vec<Match> = Vec::new();
        for rule in &self.rules {
            for m in rule.regex.find_iter(text) {
                let match_text = m.as_str();
                // 如果匹配的文本也命中排除正则，则跳过（误报过滤）
                if let Some(ref excl) = rule.exclude_regex {
                    if excl.is_match(match_text) {
                        continue;
                    }
                }
                // 运行校验器（如果存在）
                if let Some(ref validator_fn) = rule.validator_fn {
                    if !validator_fn(match_text) {
                        continue; // 校验失败，跳过此匹配
                    }
                }
                // 计算置信度
                let confidence = rule.def.base_confidence;
                matches.push(Match {
                    rule_id: rule.def.id.clone(),
                    start: m.start(),
                    end: m.end(),
                    text: match_text.to_string(),
                    priority: rule.def.priority,
                    strategy: rule.def.strategy.clone(),
                    mode: rule.def.mode.clone(),
                    confidence,
                });
            }
        }

        // 排序：优先级降 → 起始位置升 → 长度降
        matches.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.start.cmp(&b.start))
                .then_with(|| b.end.cmp(&a.end))
        });

        // 去重 + 去重叠：同位置同 rule 只保留一条，重叠的保留先出现的（高优先级 / 左侧 / 更长）
        let mut selected: Vec<Match> = Vec::new();
        for m in matches {
            // 去重：同 rule_id 同位置
            if selected.iter().any(|s| {
                s.rule_id == m.rule_id && s.start == m.start && s.end == m.end
            }) {
                continue;
            }
            // 去重叠：与已选中的任一命中范围重叠则跳过
            if selected.iter().any(|s| m.start < s.end && m.end > s.start) {
                continue;
            }
            selected.push(m);
        }

        selected
    }
}

impl Default for Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::engine::DetectionEngine for Detector {
    fn detect(&self, text: &str) -> Vec<Match> {
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

    fn reload(&mut self, dir: &Path) -> Result<usize, crate::error::DetectionError> {
        self.load_from_dir(dir)
            .map_err(|e| crate::error::DetectionError::RuleCompilation(e.to_string()))
    }

    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, crate::error::DetectionError> {
        let presets_str: Vec<&str> = presets.iter().map(|s| s.as_str()).collect();
        self.load_from_presets(base_dir, &presets_str)
            .map_err(|e| crate::error::DetectionError::RuleCompilation(e.to_string()))
    }

    fn supported_entities(&self) -> Vec<crate::entity::EntityType> {
        use std::str::FromStr;
        self.rules.iter()
            .filter_map(|r| crate::entity::EntityType::from_str(&r.def.id).ok())
            .collect()
    }

    fn name(&self) -> &str {
        "Detector"
    }
}

/// Start rule directory hot-reload. When files change, the detector is reloaded
/// using preset-based loading. Returns a watcher that must be held by the caller.
pub fn watch_rules<D: crate::engine::DetectionEngine + 'static>(
    detector: Arc<RwLock<D>>,
    dir: PathBuf,
    presets: Vec<String>,
) -> Result<notify::RecommendedWatcher> {
    let changed = Arc::new(tokio::sync::Notify::new());
    let changed_clone = changed.clone();

    let mut watcher = notify::recommended_watcher(
        move |res: std::result::Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Access(_)) {
                    return;
                }
                changed_clone.notify_one();
            }
        },
    )?;

    watcher
        .watch(&dir, RecursiveMode::Recursive)
        .with_context(|| format!("无法监听规则目录: {}", dir.display()))?;

    info!("已启动规则目录热加载: {}", dir.display());

    tokio::spawn(async move {
        loop {
            changed.notified().await;
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            info!("检测到规则文件变更，重新加载...");
            let mut d = detector.write().await;
            match d.reload_presets(&dir, &presets) {
                Ok(n) => info!("热加载完成，当前 {} 条规则", n),
                Err(e) => warn!("规则热加载失败: {}", e),
            }
        }
    });

    Ok(watcher)
}

// Re-export versioned detector types
pub use versioned::{RuleSnapshot, VersionedDetector, VersionError};
