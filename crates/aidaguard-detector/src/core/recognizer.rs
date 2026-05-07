use crate::core::result::RecognizerResult;
use aidaguard_core::EntityType;

/// The core recognizer interface — every recognizer (pattern, NLP, deny-list)
/// implements this trait.
pub trait Recognizer: Send + Sync {
    /// The entity type this recognizer can detect.
    fn entity_type(&self) -> EntityType;

    /// A human-readable name (e.g. "CreditCardRecognizer").
    fn name(&self) -> &str;

    /// Scan `text` and return all matches with confidence scores.
    fn analyze(&self, text: &str) -> Vec<RecognizerResult>;

    /// Optional: return the list of context words that, if found near a match,
    /// increase the confidence score.
    fn context_words(&self) -> &[String] {
        &[]
    }

    /// Optional: override the supported language(s) for this recognizer.
    /// Empty vec means language-agnostic.
    fn supported_languages(&self) -> &[String] {
        &[]
    }
}
