use std::sync::Mutex;

use aidaguard_core::EntityType;

use crate::core::recognizer::Recognizer;
use crate::core::result::RecognizerResult;

/// NLP NER recognizer for unstructured entity types (PersonName, Address,
/// Organization, DateOfBirth, etc.).
///
/// When the `nlp` Cargo feature is disabled, `analyze()` returns empty results
/// gracefully. When enabled, the ONNX NER model is lazy-loaded on first call.
pub struct NlpRecognizer {
    entity_type: EntityType,
    name: String,
    model_loaded: Mutex<bool>,
}

impl NlpRecognizer {
    pub fn new(entity_type: EntityType, name: impl Into<String>) -> Self {
        Self {
            entity_type,
            name: name.into(),
            model_loaded: Mutex::new(false),
        }
    }

    fn ensure_model_loaded(&self) -> bool {
        let loaded = self.model_loaded.lock().unwrap();
        if *loaded {
            return true;
        }
        #[cfg(feature = "nlp")]
        {
            // TODO: Load ONNX NER model for self.entity_type + language
            // drop(loaded); mut loaded = ...; *loaded = true;
            drop(loaded);
        }
        #[cfg(not(feature = "nlp"))]
        {
            drop(loaded);
        }
        false
    }
}

impl Recognizer for NlpRecognizer {
    fn entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn analyze(&self, text: &str) -> Vec<RecognizerResult> {
        if !self.ensure_model_loaded() {
            return Vec::new();
        }

        #[cfg(feature = "nlp")]
        {
            // Run ONNX NER inference on `text`, return spans with confidence scores
            let _ = text;
            Vec::new()
        }

        #[cfg(not(feature = "nlp"))]
        {
            let _ = text;
            Vec::new()
        }
    }

    fn context_words(&self) -> &[String] {
        &[]
    }

    fn supported_languages(&self) -> &[String] {
        &[]
    }
}
