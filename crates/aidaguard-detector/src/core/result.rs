use aidaguard_core::detector::{Match, Mode, Strategy};
use aidaguard_core::EntityType;
use serde::{Deserialize, Serialize};

/// A single detection result from one recognizer, carrying entity type and confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizerResult {
    /// Entity type (CreditCard, Email, PersonName, etc.)
    pub entity_type: EntityType,
    /// Start byte offset in the original text
    pub start: usize,
    /// End byte offset (exclusive) in the original text
    pub end: usize,
    /// The matched text
    pub text: String,
    /// Confidence score, normalized to 0.0..=1.0
    pub score: f64,
    /// Recognizer name for debugging
    pub recognizer_name: String,
}

impl RecognizerResult {
    /// Convert to legacy Match for use with replacer and storage subsystems.
    pub fn to_legacy_match(&self, strategy: Strategy, mode: Mode) -> Match {
        Match {
            rule_id: self.entity_type.to_string(),
            start: self.start,
            end: self.end,
            text: self.text.clone(),
            priority: 100,
            strategy,
            mode,
            confidence: Some(self.score),
        }
    }

    /// Create from a regex match with minimum information.
    pub fn from_regex_match(
        entity_type: EntityType,
        start: usize,
        end: usize,
        text: &str,
        score: f64,
        recognizer_name: &str,
    ) -> Self {
        Self {
            entity_type,
            start,
            end,
            text: text.to_string(),
            score,
            recognizer_name: recognizer_name.to_string(),
        }
    }
}
