use aidaguard_core::detector::CompiledRule;
use aidaguard_core::EntityType;
use regex::Regex;

use crate::core::recognizer::Recognizer;
use crate::core::result::RecognizerResult;
use crate::validation::context::LemmaContextAwareEnhancer;

/// A recognizer dynamically generated from a YAML rule definition.
///
/// Unlike hardcoded `PatternRecognizer` instances, this is created at runtime
/// from `CompiledRule` data loaded from YAML files. It supports:
/// - Regex matching from YAML `pattern` field
/// - Exclude regex filtering from YAML `exclude` field
/// - Checksum validation from `validator_fn` (wired via ValidatorRegistry)
/// - Context word boosting from YAML `context_words` field
/// - Base confidence from YAML `base_confidence` field
pub struct YamlRecognizer {
    entity_type: EntityType,
    recognizer_name: String,
    pattern: Regex,
    exclude_regex: Option<Regex>,
    base_confidence: f64,
    context_words: Vec<String>,
    context_enhancer: LemmaContextAwareEnhancer,
}

impl YamlRecognizer {
    /// Create a YamlRecognizer from a compiled rule.
    ///
    /// Note: The `validator_fn` from `CompiledRule` is NOT used here.
    /// Validation is handled by the `Detector` layer after regex matching.
    /// This separation allows the same rule to work in both the legacy
    /// Detector path and the Recognizer pipeline.
    pub fn from_compiled_rule(rule: &CompiledRule) -> Self {
        let entity_type = EntityType::Custom(rule.def.id.clone());
        let recognizer_name = format!("YamlRecognizer_{}", rule.def.id);

        Self {
            entity_type,
            recognizer_name,
            pattern: rule.regex.clone(),
            exclude_regex: rule.exclude_regex.clone(),
            base_confidence: rule.def.base_confidence.unwrap_or(0.5),
            context_words: rule.def.context_words.clone(),
            context_enhancer: LemmaContextAwareEnhancer::default_window(),
        }
    }

    /// Create a YamlRecognizer from a CompiledRule with an explicit entity type.
    pub fn with_entity_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = entity_type;
        self
    }
}

impl Recognizer for YamlRecognizer {
    fn entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    fn name(&self) -> &str {
        &self.recognizer_name
    }

    fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = Vec::new();

        for m in self.pattern.find_iter(text) {
            let matched_text = m.as_str();

            // Exclude regex filter
            if let Some(ref excl) = self.exclude_regex {
                if excl.is_match(matched_text) {
                    continue;
                }
            }

            let mut result = RecognizerResult {
                entity_type: self.entity_type.clone(),
                start: m.start(),
                end: m.end(),
                text: matched_text.to_string(),
                score: self.base_confidence,
                recognizer_name: self.recognizer_name.clone(),
            };

            // Context word boost
            if !self.context_words.is_empty() {
                result.score = self.context_enhancer.enhance(
                    &result,
                    text,
                    &self.context_words,
                );
            }

            results.push(result);
        }

        results
    }

    fn context_words(&self) -> &[String] {
        &self.context_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidaguard_core::detector::{RuleDef, Strategy, Mode};

    fn make_rule(id: &str, pattern: &str, context_words: Vec<&str>, base_confidence: Option<f64>) -> CompiledRule {
        let def = RuleDef {
            id: id.to_string(),
            name: id.to_string(),
            pattern: pattern.to_string(),
            exclude: None,
            enabled: true,
            strategy: Strategy::Placeholder,
            mode: Mode::Filter,
            priority: 100,
            compliance: vec![],
            validator: None,
            context_words: context_words.into_iter().map(|s| s.to_string()).collect(),
            base_confidence,
            region: None,
            source: "system".to_string(),
        };
        let regex = aidaguard_core::detector::compile_regex(&def.pattern).unwrap();
        CompiledRule {
            def,
            regex,
            exclude_regex: None,
            validator_fn: None,
        }
    }

    #[test]
    fn test_basic_detection() {
        let rule = make_rule("phone_cn", r"1[3-9]\d{9}", vec![], None);
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        let results = recognizer.analyze("我的手机号是13812345678，请联系我");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "13812345678");
        assert_eq!(results[0].score, 0.5); // default base_confidence
    }

    #[test]
    fn test_base_confidence() {
        let rule = make_rule("test", r"\d{6}", vec![], Some(0.8));
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        let results = recognizer.analyze("验证码是123456");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 0.8);
    }

    #[test]
    fn test_context_word_boost() {
        let rule = make_rule("credit_card", r"\d{16}", vec!["card", "credit", "visa"], Some(0.4));
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        let text = "my credit card number is 4532015112830366";
        let results = recognizer.analyze(text);
        assert_eq!(results.len(), 1);
        // Score should be boosted above base_confidence due to context words
        assert!(results[0].score > 0.4);
    }

    #[test]
    fn test_no_context_boost_without_words() {
        let rule = make_rule("digits", r"\d{6}", vec![], Some(0.5));
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        let results = recognizer.analyze("验证码是123456");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 0.5);
    }

    #[test]
    fn test_entity_type_is_custom() {
        let rule = make_rule("my_custom_id", r"ABC\d{4}", vec![], None);
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        assert!(matches!(recognizer.entity_type(), EntityType::Custom(_)));
    }

    #[test]
    fn test_recognizer_name() {
        let rule = make_rule("phone_cn", r"1[3-9]\d{9}", vec![], None);
        let recognizer = YamlRecognizer::from_compiled_rule(&rule);

        assert_eq!(recognizer.name(), "YamlRecognizer_phone_cn");
    }
}
