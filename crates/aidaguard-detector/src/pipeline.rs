use std::path::Path;

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
}

impl AnalyzerEngine {
    pub fn builder() -> AnalyzerEngineBuilder {
        AnalyzerEngineBuilder::default()
    }

    /// Run detection across the full pipeline:
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

    /// Return the raw recognizer results converted to legacy Match format.
    fn scan_as_matches(&self, text: &str) -> Vec<Match> {
        let recognizer_results = self.scan(text);
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
}

/// Builder for [`AnalyzerEngine`].
#[derive(Default)]
pub struct AnalyzerEngineBuilder {
    rules_base_dir: Option<String>,
    rules_presets: Vec<String>,
    min_confidence: f64,
    load_predefined: bool,
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
    /// (e.g. `"global"`, `"regions/cn"`, `"domains/medical"`).
    pub fn with_rules_presets(mut self, base_dir: impl Into<String>, presets: &[&str]) -> Self {
        self.rules_base_dir = Some(base_dir.into());
        self.rules_presets = presets.iter().map(|s| s.to_string()).collect();
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

    /// Build the engine.
    pub fn build(self) -> Result<AnalyzerEngine, anyhow::Error> {
        let mut registry = RecognizerRegistry::new();

        if self.load_predefined {
            registry.load_predefined();
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
        })
    }
}
