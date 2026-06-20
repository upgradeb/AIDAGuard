use std::path::Path;

use aidaguard_core::config::Config;
use aidaguard_core::detector::{Match, Strategy, Mode};
use aidaguard_core::DetectionEngine;
use aidaguard_core::entity::EntityType;
use aidaguard_core::error::DetectionError;

use crate::core::confidence::ConfidenceScorer;
use crate::core::recognizer_registry::RecognizerRegistry;
use crate::core::result::RecognizerResult;

/// The main detection pipeline coordinating YAML rules and pattern recognizers.
///
/// All YAML rules are loaded as YamlRecognizers in the registry, eliminating the
/// legacy dual-path (Detector + registry) that caused double-counting.
///
/// Build with [`AnalyzerEngineBuilder`].
pub struct AnalyzerEngine {
    registry: RecognizerRegistry,
    min_confidence: f64,
}

impl AnalyzerEngine {
    pub fn builder() -> AnalyzerEngineBuilder {
        AnalyzerEngineBuilder::default()
    }

    /// Run detection across the full pipeline (sequential):
    /// 1. Pattern recognizers (regex + checksum + context)
    /// 2. Overlap resolution
    /// 3. Confidence filtering
    pub fn scan(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = self.registry.analyze_all(text);

        results = ConfidenceScorer::resolve_overlaps(results);

        results.retain(|r| r.score >= self.min_confidence);

        results
    }

    /// Run detection in parallel using rayon.
    pub fn scan_parallel(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = self.registry.analyze_all_parallel(text);

        results = ConfidenceScorer::resolve_overlaps(results);

        results.retain(|r| r.score >= self.min_confidence);

        results
    }

    /// Return the raw recognizer results converted to legacy Match format,
    /// using the full detection pipeline (overlap resolution + confidence filtering).
    fn scan_as_matches(&self, text: &str) -> Vec<Match> {
        let recognizer_results = self.scan(text);
        recognizer_results
            .iter()
            .map(|r| r.to_legacy_match(Strategy::Placeholder, Mode::Filter))
            .collect()
    }
}

impl DetectionEngine for AnalyzerEngine {
    fn detect(&self, text: &str) -> Vec<Match> {
        self.scan_as_matches(text)
    }

    fn detect_parallel(&self, text: &str) -> Vec<Match> {
        // Consistent with detect() — full pipeline with overlap + confidence filtering
        let results = self.scan_parallel(text);
        results
            .into_iter()
            .map(|r| r.to_legacy_match(Strategy::Placeholder, Mode::Filter))
            .collect()
    }

    fn rule_count(&self) -> usize {
        self.registry.recognizer_count()
    }

    fn rule_name(&self, id: &str) -> Option<&str> {
        self.registry.entity_name(id)
    }

    fn rule_ids(&self) -> Vec<String> {
        self.registry.recognizer_ids()
    }

    fn reload(&mut self, dir: &Path) -> Result<usize, DetectionError> {
        let mut registry = RecognizerRegistry::new();
        registry.load_predefined();
        registry.load_from_rules_dir(dir)
            .map_err(|e| DetectionError::RuleCompilation(e.to_string()))?;
        self.registry = registry;
        Ok(self.registry.recognizer_count())
    }

    fn reload_presets(&mut self, base_dir: &Path, presets: &[String]) -> Result<usize, DetectionError> {
        let mut registry = RecognizerRegistry::new();
        registry.load_predefined();
        let presets_refs: Vec<&str> = presets.iter().map(|s| s.as_str()).collect();
        registry.load_from_rules_presets(base_dir, &presets_refs)
            .map_err(|e| DetectionError::RuleCompilation(e.to_string()))?;
        self.registry = registry;
        Ok(self.registry.recognizer_count())
    }

    fn supported_entities(&self) -> Vec<EntityType> {
        self.registry.get_supported_entities()
    }

    fn name(&self) -> &str {
        "AnalyzerEngine"
    }
}

/// Builder for [`AnalyzerEngine`].
pub struct AnalyzerEngineBuilder {
    rules_base_dir: Option<String>,
    rules_presets: Vec<String>,
    min_confidence: f64,
    load_predefined: bool,
    nlp_enabled: bool,
    nlp_language: String,
}

impl Default for AnalyzerEngineBuilder {
    fn default() -> Self {
        Self {
            rules_base_dir: None,
            rules_presets: Vec::new(),
            min_confidence: 0.0,
            load_predefined: false,
            nlp_enabled: false,
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
    pub fn with_rules_presets(mut self, base_dir: impl Into<String>, presets: &[&str]) -> Self {
        self.rules_base_dir = Some(base_dir.into());
        self.rules_presets = presets.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Load rules according to the region and industry configuration.
    pub fn with_config_rules(mut self, config: &Config) -> Self {
        self.rules_base_dir = Some(config.rules_dir.clone());
        self.rules_presets = config.rule_presets();
        self
    }

    /// Set the minimum confidence threshold.
    pub fn with_min_confidence(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Register all built-in pattern recognizers.
    pub fn with_all_pattern_recognizers(mut self) -> Self {
        self.load_predefined = true;
        self
    }

    /// Load YAML rules as recognizers integrated into the recognizer pipeline
    /// with validator and context word support (now always on when dir is set).
    pub fn with_yaml_as_recognizers(self) -> Self {
        self
    }

    /// Set the NLP model language.
    pub fn with_nlp_language(mut self, language: impl Into<String>) -> Self {
        self.nlp_language = language.into();
        self
    }

    /// Apply NLP settings from config.
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
    ///
    /// Creates a single RecognizerRegistry containing both built-in pattern
    /// recognizers and YAML rule recognizers. The legacy Detector dual-path
    /// has been removed.
    pub fn build(self) -> Result<AnalyzerEngine, anyhow::Error> {
        let mut registry = RecognizerRegistry::new();

        if self.load_predefined {
            registry.load_predefined();
            if self.nlp_enabled {
                registry.load_nlp_recognizers(&self.nlp_language);
            }
        }

        if let Some(base_dir) = self.rules_base_dir {
            let base = Path::new(&base_dir);
            if self.rules_presets.is_empty() {
                registry.load_from_rules_dir(base)?;
            } else {
                let presets_refs: Vec<&str> = self.rules_presets.iter().map(|s| s.as_str()).collect();
                registry.load_from_rules_presets(base, &presets_refs)?;
            }
        }

        Ok(AnalyzerEngine {
            registry,
            min_confidence: self.min_confidence,
        })
    }
}
