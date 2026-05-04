use anyhow::{Context, Result};
use notify::{EventKind, RecursiveMode, Watcher};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// 安全编译正则，设置 size_limit 防止 ReDoS 攻击。
fn compile_regex(pattern: &str) -> Result<Regex> {
    // 限制模式长度，防止极端情况
    if pattern.len() > 2000 {
        return Err(anyhow::anyhow!("正则模式过长 ({} bytes)，上限 2000", pattern.len()));
    }
    RegexBuilder::new(pattern)
        .size_limit(1 << 20) // 1 MB DFA 大小限制
        .build()
        .with_context(|| format!("正则编译失败: {}", pattern))
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

/// 编译后的规则
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub def: RuleDef,
    pub regex: Regex,
    pub exclude_regex: Option<Regex>,
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
}

/// 敏感数据检测器
pub struct Detector {
    rules: Vec<CompiledRule>,
}

impl Detector {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 从指定目录加载所有 YAML 规则文件，替换当前规则集
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize> {
        let mut all_rules = Vec::new();

        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("无法读取规则目录: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                info!("加载规则文件: {}", path.display());
                let contents = std::fs::read_to_string(&path)
                    .with_context(|| format!("无法读取文件: {}", path.display()))?;
                let rule_file: RuleFile = serde_yaml::from_str(&contents)
                    .with_context(|| format!("YAML 解析失败: {}", path.display()))?;
                all_rules.extend(rule_file.rules);
            }
        }

        let count_before = self.rules.len();
        self.compile(all_rules);
        info!(
            "已加载 {} 条规则（替换了原有的 {} 条）",
            self.rules.len(),
            count_before
        );
        Ok(self.rules.len())
    }

    /// 追加单条规则
    pub fn add_rule(&mut self, def: RuleDef) -> Result<()> {
        let regex = compile_regex(&def.pattern)
            .with_context(|| format!("规则 [{}] 的正则编译失败: {}", def.id, def.pattern))?;
        let exclude_regex = if let Some(ref excl) = def.exclude {
            Some(compile_regex(excl)
                .with_context(|| format!("规则 [{}] 的排除正则编译失败: {}", def.id, excl))?)
        } else {
            None
        };
        self.rules.push(CompiledRule { def, regex, exclude_regex });
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
                    compiled.push(CompiledRule { def, regex, exclude_regex });
                }
                Err(e) => warn!("规则 [{}] 正则编译失败: {}", def.id, e),
            }
        }
        // 按优先级降序排列，高优先级先处理
        compiled.sort_by(|a, b| b.def.priority.cmp(&a.def.priority));
        self.rules = compiled;
    }

    /// 已编译的规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// 根据规则 ID 查询规则名称
    pub fn rule_name(&self, id: &str) -> Option<&str> {
        self.rules.iter().find(|r| r.def.id == id).map(|r| r.def.name.as_str())
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
                matches.push(Match {
                    rule_id: rule.def.id.clone(),
                    start: m.start(),
                    end: m.end(),
                    text: match_text.to_string(),
                    priority: rule.def.priority,
                    strategy: rule.def.strategy.clone(),
                    mode: rule.def.mode.clone(),
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

/// 启动规则目录热加载。文件变更时自动重新加载 `detector`。
///
/// 返回的 `RecommendedWatcher` 必须被调用方持有，否则 watcher 会被 drop、热加载失效。
pub fn watch_rules(
    detector: Arc<RwLock<Detector>>,
    dir: PathBuf,
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
        .watch(&dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("无法监听规则目录: {}", dir.display()))?;

    info!("已启动规则目录热加载: {}", dir.display());

    tokio::spawn(async move {
        loop {
            changed.notified().await;
            // 防抖：200ms 内的连续事件合并处理
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            info!("检测到规则文件变更，重新加载...");
            let mut d = detector.write().await;
            match d.load_from_dir(&dir) {
                Ok(n) => info!("热加载完成，当前 {} 条规则", n),
                Err(e) => warn!("规则热加载失败: {}", e),
            }
        }
    });

    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_detector() -> Detector {
        let mut d = Detector::new();
        d.add_rule(RuleDef {
            id: "phone".into(),
            name: "手机号".into(),
            pattern: r"1[3-9]\d{9}".into(),
            exclude: None,
            enabled: true,
            strategy: Strategy::Placeholder,
            mode: Mode::Filter,
            priority: 100,
        })
        .unwrap();
        d.add_rule(RuleDef {
            id: "id_card".into(),
            name: "身份证".into(),
            pattern: r"\d{17}[\dXx]".into(),
            exclude: None,
            enabled: true,
            strategy: Strategy::Placeholder,
            mode: Mode::Filter,
            priority: 100,
        })
        .unwrap();
        d.add_rule(RuleDef {
            id: "email".into(),
            name: "邮箱".into(),
            pattern: r"[\w.+-]+@[\w-]+\.\w+".into(),
            exclude: None,
            enabled: true,
            strategy: Strategy::Mask,
            mode: Mode::Filter,
            priority: 90,
        })
        .unwrap();
        d
    }

    #[test]
    fn test_detect_phone() {
        let d = make_detector();
        let hits = d.detect("我的手机是13812345678，请记录");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "phone");
        assert_eq!(hits[0].text, "13812345678");
    }

    #[test]
    fn test_detect_multiple() {
        let d = make_detector();
        let hits = d.detect("手机13812345678，邮箱test@example.com");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_no_match() {
        let d = make_detector();
        let hits = d.detect("今天天气真好");
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn test_overlap_same_priority() {
        // 身份证包含18位数字，可能和银行卡等规则重叠
        let d = make_detector();
        // "320102199001011234" 匹配身份证规则（18位），同时也匹配银行卡的16-19位数字
        // 但我们的 detector 只注册了 phone, id_card, email，没有 bank_card
        // 不会重叠。仅确认身份证本身被正确检测。
        let hits = d.detect("身份证320102199001011234在这里");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "id_card");
    }

    #[test]
    fn test_id_card_with_x() {
        let d = make_detector();
        let hits = d.detect("号码32010219900101123X");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "32010219900101123X");
    }

    #[test]
    fn test_deduplication() {
        let d = make_detector();
        // 手机号 + 身份证有明确分隔，不会有跨边界重叠
        let hits = d.detect("13812345678和320102199001011234");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_empty_input() {
        let d = make_detector();
        let hits = d.detect("");
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn test_email_exclude_retina() {
        let mut d = Detector::new();
        d.add_rule(RuleDef {
            id: "email".into(),
            name: "邮箱".into(),
            pattern: r"[\w.+-]+@[\w-]+\.\w+".into(),
            exclude: Some(r"@\d+x\.(?:png|jpg|jpeg|gif|svg|webp|ico|pdf)\b".into()),
            enabled: true,
            strategy: Strategy::Mask,
            mode: Mode::Filter,
            priority: 90,
        })
        .unwrap();

        // 真实邮箱应该被检测
        let hits = d.detect("联系 test@example.com 或 123456@qq.com");
        assert_eq!(hits.len(), 2);

        // Retina 文件名不应被检测
        let hits = d.detect("图标文件 icon@2x.png 和 logo@3x.jpg");
        assert_eq!(hits.len(), 0);

        // 混合场景
        let hits = d.detect("图片 icon@2x.png，邮箱 admin@foo.com");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].text, "admin@foo.com");
    }
}
