use std::sync::Arc;

use aidaguard_core::EntityType;
use regex::Regex;

use crate::core::recognizer::Recognizer;
use crate::core::result::RecognizerResult;
use crate::validation::context::LemmaContextAwareEnhancer;

type ValidatorFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// A regex-based recognizer with optional checksum validation and context-word boosting.
///
/// Detection flow: broad regex → checksum filter → context enhance.
pub struct PatternRecognizer {
    entity_type: EntityType,
    name: String,
    pattern: Regex,
    base_confidence: f64,
    validator: Option<ValidatorFn>,
    context_words: Vec<String>,
    context_enhancer: LemmaContextAwareEnhancer,
}

impl PatternRecognizer {
    pub fn new(
        entity_type: EntityType,
        name: impl Into<String>,
        pattern: Regex,
        base_confidence: f64,
    ) -> Self {
        Self {
            entity_type,
            name: name.into(),
            pattern,
            base_confidence,
            validator: None,
            context_words: Vec::new(),
            context_enhancer: LemmaContextAwareEnhancer::default_window(),
        }
    }

    /// Attach a checksum validator. When set, regex matches that fail validation are discarded.
    pub fn with_validator(mut self, f: ValidatorFn) -> Self {
        self.validator = Some(f);
        self
    }

    /// Set context words for confidence boosting.
    pub fn with_context_words(mut self, words: Vec<impl Into<String>>) -> Self {
        self.context_words = words.into_iter().map(|w| w.into()).collect();
        self
    }
}

impl Recognizer for PatternRecognizer {
    fn entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = Vec::new();

        for caps in self.pattern.captures_iter(text) {
            let Some(m) = caps.get(0) else { continue };
            let matched_text = m.as_str();
            let start = m.start();
            let end = m.end();

            // Step 1: optional checksum validation
            if let Some(ref validator) = self.validator {
                if !validator(matched_text) {
                    continue;
                }
            }

            // Step 2: build result
            let mut result = RecognizerResult {
                entity_type: self.entity_type.clone(),
                start,
                end,
                text: matched_text.to_string(),
                score: self.base_confidence,
                recognizer_name: self.name.clone(),
            };

            // Step 3: context word boost
            if !self.context_words.is_empty() {
                result.score =
                    self.context_enhancer
                        .enhance(&result, text, &self.context_words);
            }

            results.push(result);
        }

        results
    }

    fn context_words(&self) -> &[String] {
        &self.context_words
    }
}
