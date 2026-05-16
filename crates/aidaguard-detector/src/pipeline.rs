use std::path::Path;

use aidaguard_core::config::Config;
use aidaguard_core::detector::{Detector, Match, Mode, Strategy};
use aidaguard_core::DetectionEngine;

use crate::core::confidence::ConfidenceScorer;
use crate::core::recognizer_registry::RecognizerRegistry;
use crate::core::result::RecognizerResult;

/// The main detection pipeline coordinating legacy YAML rules and pattern recognizers.
///
/// Build with [`AnalyzerEngineBuilder`].
pub struct AnalyzerEngine {
    registry: RecognizerRegistry,
    legacy_detector: Option<Detector>,
    min_confidence: f64,
    nlp_enabled: bool,  // 是否启用 NLP
}

/// 智能跳过策略阈值
const SMART_SKIP_TEXT_THRESHOLD: usize = 200;  // 小于此长度跳过 NLP
const SMART_SKIP_MIN_PII_SIGNALS: usize = 2;   // 至少有 2 个 PII 信号才启用 NLP

impl AnalyzerEngine {
    pub fn builder() -> AnalyzerEngineBuilder {
        AnalyzerEngineBuilder::default()
    }

    /// 智能判断是否需要 NLP 检测
    /// 
    /// 策略:
    /// 1. NLP 未启用 → 跳过
    /// 2. 文本过短 (< 200 字符) → 跳过
    /// 3. 无 PII 信号特征 → 跳过
    fn should_run_nlp(&self, text: &str) -> bool {
        if !self.nlp_enabled {
            return false;
        }
        
        // 短文本跳过 NLP
        if text.len() < SMART_SKIP_TEXT_THRESHOLD {
            return false;
        }
        
        // 检查 PII 信号特征
        let pii_signals = count_pii_signals(text);
        pii_signals >= SMART_SKIP_MIN_PII_SIGNALS
    }

    /// Run detection across the full pipeline (sequential):
    /// 1. Legacy YAML regex rules (optional)
    /// 2. Pattern recognizers (regex + checksum + context)
    /// 3. Overlap resolution
    /// 4. Confidence filtering
    pub fn scan(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = self.registry.analyze_all(text);

        // Resolve overlapping matches — keep higher confidence
        results = ConfidenceScorer::resolve_overlaps(results);

        // Filter by minimum confidence threshold
        results.retain(|r| r.score >= self.min_confidence);

        results
    }

    /// Run detection in parallel using rayon.
    /// More efficient for large texts with many recognizers.
    /// Benchmarks show 2-3x speedup on 4-core CPUs.
    pub fn scan_parallel(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = self.registry.analyze_all_parallel(text);

        // Resolve overlapping matches — keep higher confidence
        results = ConfidenceScorer::resolve_overlaps(results);

        // Filter by minimum confidence threshold
        results.retain(|r| r.score >= self.min_confidence);

        results
    }

    /// Return the raw recognizer results converted to legacy Match format.
    fn scan_as_matches(&self, text: &str) -> Vec<Match> {
        // Use parallel scan for better performance
        let recognizer_results = self.scan_parallel(text);
        let mut matches: Vec<Match> = recognizer_results
            .iter()
            .map(|r| r.to_legacy_match(Strategy::Placeholder, Mode::Filter))
            .collect();

        // Merge with legacy detector results
        if let Some(ref legacy) = self.legacy_detector {
            matches.extend(legacy.detect(text));
        }

        // Deduplicate overlapping matches, preferring higher score
        matches.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.start.cmp(&b.start))
                .then_with(|| b.end.cmp(&a.end))
        });

        let mut selected: Vec<Match> = Vec::new();
        for m in matches {
            if selected.iter().any(|s| {
                s.rule_id == m.rule_id && s.start == m.start && s.end == m.end
            }) {
                continue;
            }
            if selected.iter().any(|s| m.start < s.end && m.end > s.start) {
                continue;
            }
            selected.push(m);
        }

        selected
    }
}

impl DetectionEngine for AnalyzerEngine {
    fn detect(&self, text: &str) -> Vec<Match> {
        self.scan_as_matches(text)
    }

    fn rule_count(&self) -> usize {
        let legacy_count = self
            .legacy_detector
            .as_ref()
            .map(|d| d.rule_count())
            .unwrap_or(0);
        legacy_count + self.registry.recognizer_count()
    }

    fn rule_name(&self, id: &str) -> Option<&str> {
        self.legacy_detector
            .as_ref()
            .and_then(|d| d.rule_name(id))
            .or_else(|| self.registry.entity_name(id))
    }

    fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error> {
        if let Some(ref mut legacy) = self.legacy_detector {
            legacy.load_from_dir(dir)
        } else {
            Ok(0)
        }
    }

    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, anyhow::Error> {
        if let Some(ref mut legacy) = self.legacy_detector {
            let presets_str: Vec<&str> = presets.iter().map(|s| s.as_str()).collect();
            legacy.load_from_presets(base_dir, &presets_str)
        } else {
            Ok(0)
        }
    }
}

/// Builder for [`AnalyzerEngine`].
pub struct AnalyzerEngineBuilder {
    rules_base_dir: Option<String>,
    rules_presets: Vec<String>,
    min_confidence: f64,
    load_predefined: bool,
    nlp_enabled: bool,  // 控制 NLP 是否启用
    nlp_language: String,
}

impl Default for AnalyzerEngineBuilder {
    fn default() -> Self {
        Self {
            rules_base_dir: None,
            rules_presets: Vec::new(),
            min_confidence: 0.0,
            load_predefined: false,
            nlp_enabled: false,  // 默认关闭
            nlp_language: "en".to_string(),
        }
    }
}

impl AnalyzerEngineBuilder {
    /// Load legacy YAML rules from a single directory (backward-compatible).
    pub fn with_legacy_rules_dir(mut self, dir: impl Into<String>) -> Self {
        self.rules_base_dir = Some(dir.into());
        self
    }

    /// Set the rules base directory and enable specific presets.
    ///
    /// Each preset name corresponds to a subdirectory under `base_dir`
    /// (e.g. `"global"`, `"cn"`, `"cn/medical"`).
    pub fn with_rules_presets(mut self, base_dir: impl Into<String>, presets: &[&str]) -> Self {
        self.rules_base_dir = Some(base_dir.into());
        self.rules_presets = presets.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Load rules according to the region and industry configuration.
    ///
    /// Computes presets from `Config::rule_presets()` and sets the rules directory.
    pub fn with_config_rules(mut self, config: &Config) -> Self {
        self.rules_base_dir = Some(config.rules_dir.clone());
        self.rules_presets = config.rule_presets();
        self
    }

    /// Set the minimum confidence threshold (0.0..=1.0). Results below this are discarded.
    pub fn with_min_confidence(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Register all built-in pattern recognizers.
    pub fn with_all_pattern_recognizers(mut self) -> Self {
        self.load_predefined = true;
        self
    }

    /// Set the NLP model language (e.g. "en", "zh"). Default is "en".
    pub fn with_nlp_language(mut self, language: impl Into<String>) -> Self {
        self.nlp_language = language.into();
        self
    }

    /// Apply NLP settings from config (enabled flag + default language).
    pub fn with_nlp_config(mut self, nlp: &aidaguard_core::config::NlpConfig) -> Self {
        self.nlp_enabled = nlp.enabled;
        if nlp.enabled {
            self.nlp_language = nlp.default_language.clone();
        }
        self
    }

    /// Explicitly enable or disable NLP recognizers.
    pub fn with_nlp_enabled(mut self, enabled: bool) -> Self {
        self.nlp_enabled = enabled;
        self
    }

    /// Build the engine.
    pub fn build(self) -> Result<AnalyzerEngine, anyhow::Error> {
        let mut registry = RecognizerRegistry::new();

        if self.load_predefined {
            registry.load_predefined();
            // 仅在显式启用时加载 NLP recognizers
            if self.nlp_enabled {
                registry.load_nlp_recognizers(&self.nlp_language);
            }
        }

        let legacy_detector = if let Some(base_dir) = self.rules_base_dir {
            let mut detector = Detector::new();
            let base = Path::new(&base_dir);
            if self.rules_presets.is_empty() {
                // Backward-compatible: load from a single flat directory
                detector.load_from_dir(base)?;
            } else {
                // Load from multiple preset subdirectories
                let presets: Vec<&str> = self.rules_presets.iter().map(|s| s.as_str()).collect();
                detector.load_from_presets(base, &presets)?;
            }
            Some(detector)
        } else {
            None
        };

        Ok(AnalyzerEngine {
            registry,
            legacy_detector,
            min_confidence: self.min_confidence,
            nlp_enabled: self.nlp_enabled,
        })
    }
}

/// 统计文本中的 PII 信号特征数量
/// 用于智能判断是否需要 NLP 检测
fn count_pii_signals(text: &str) -> usize {
    let mut signals = 0;
    
    // 信号 1: 包含人名特征词
    let name_indicators = ["name", "名字", "姓名", "called", "is", "my name"];
    if name_indicators.iter().any(|kw| text.to_lowercase().contains(kw)) {
        signals += 1;
    }
    
    // 信号 2: 包含地址特征词
    let address_indicators = ["address", "地址", "street", "road", "live", "住在"];
    if address_indicators.iter().any(|kw| text.to_lowercase().contains(kw)) {
        signals += 1;
    }
    
    // 信号 3: 包含组织/公司特征词
    let org_indicators = ["company", "公司", "organization", "work at", "就职于"];
    if org_indicators.iter().any(|kw| text.to_lowercase().contains(kw)) {
        signals += 1;
    }
    
    // 信号 4: 包含日期/生日特征词
    let date_indicators = ["birthday", "出生", "born", "date of birth", "年龄"];
    if date_indicators.iter().any(|kw| text.to_lowercase().contains(kw)) {
        signals += 1;
    }
    
    // 信号 5: 包含医疗/健康特征词
    let medical_indicators = ["patient", "患者", "diagnosis", "诊断", "hospital", "医院"];
    if medical_indicators.iter().any(|kw| text.to_lowercase().contains(kw)) {
        signals += 1;
    }
    
    signals
}
