use std::collections::HashMap;
use std::sync::Mutex;

use aidaguard_core::EntityType;

use crate::core::recognizer::Recognizer;
use crate::core::result::RecognizerResult;

#[cfg(feature = "nlp")]
mod registry;
#[cfg(feature = "nlp")]
mod engine;
#[cfg(feature = "nlp")]
pub mod mapping;

#[cfg(feature = "nlp")]
use engine::{NlpEngine, RawNerSpan};
#[cfg(feature = "nlp")]
use mapping::LabelMapping;
#[cfg(feature = "nlp")]
use registry::ModelRegistry;

/// Simple per-text inference cache to avoid re-running the NER model for each
/// of the 10 NlpRecognizer instances on the same input.
#[cfg(feature = "nlp")]
struct InferenceCache {
    entries: Mutex<HashMap<CacheKey, Option<std::sync::Arc<Vec<RawNerSpan>>>>>,
}

#[cfg(feature = "nlp")]
#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    text_hash: u64,
    language: String,
}

#[cfg(feature = "nlp")]
impl InferenceCache {
    fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    fn global() -> &'static Self {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<InferenceCache> = OnceLock::new();
        INSTANCE.get_or_init(InferenceCache::new)
    }

    fn get_or_run<E>(
        &self,
        text: &str,
        language: &str,
        run_fn: impl FnOnce() -> Result<Vec<RawNerSpan>, E>,
    ) -> Result<Option<std::sync::Arc<Vec<RawNerSpan>>>, E> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let key = CacheKey { text_hash: hasher.finish(), language: language.to_string() };

        {
            let cache = self.entries.lock().unwrap();
            if let Some(entry) = cache.get(&key) {
                return Ok(entry.clone());
            }
        }

        match run_fn() {
            Ok(spans) => {
                let result = std::sync::Arc::new(spans);
                let mut cache = self.entries.lock().unwrap();
                // Keep cache bounded — purge if over 512 entries
                if cache.len() > 512 {
                    cache.clear();
                }
                cache.insert(key, Some(result.clone()));
                Ok(Some(result))
            }
            Err(_e) => {
                let mut cache = self.entries.lock().unwrap();
                cache.insert(key, None);
                Ok(None)
            }
        }
    }
}

/// NLP NER recognizer for unstructured entity types (PersonName, Address,
/// Organization, DateOfBirth, etc.).
///
/// When the `nlp` Cargo feature is disabled, `analyze()` returns empty results
/// gracefully. When enabled, the BERT NER model is lazy-loaded on first call
/// and downloaded from HuggingFace Hub if not already cached locally.
pub struct NlpRecognizer {
    entity_type: EntityType,
    name: String,
    language: String,
    model_loaded: Mutex<bool>,
}

impl NlpRecognizer {
    pub fn new(entity_type: EntityType, name: impl Into<String>) -> Self {
        Self {
            entity_type,
            name: name.into(),
            language: "en".to_string(),
            model_loaded: Mutex::new(false),
        }
    }

    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }

    fn ensure_model_loaded(&self) -> bool {
        if *self.model_loaded.lock().unwrap() {
            return true;
        }

        #[cfg(feature = "nlp")]
        {
            match ModelRegistry::global().get_or_load(&self.language) {
                Ok(_) => {
                    *self.model_loaded.lock().unwrap() = true;
                    true
                }
                Err(e) => {
                    tracing::warn!("Failed to load NLP model for '{}': {}", self.language, e);
                    false
                }
            }
        }

        #[cfg(not(feature = "nlp"))]
        { false }
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
            let model = match ModelRegistry::global().get_loaded(&self.language) {
                Some(m) => m,
                None => return Vec::new(),
            };

            let spans = InferenceCache::global().get_or_run(text, &self.language, || {
                let engine = NlpEngine::new(model.clone());
                engine.run(text)
            });

            let spans = match spans {
                Ok(Some(s)) => s,
                _ => return Vec::new(),
            };

            let mapping = LabelMapping::new();
            let mut results = Vec::new();

            for span in spans.iter() {
                let Some(mapped_type) = mapping.entity_type_for(&span.label) else {
                    continue;
                };
                if mapped_type != self.entity_type {
                    continue;
                }

                results.push(RecognizerResult {
                    entity_type: self.entity_type.clone(),
                    start: span.start,
                    end: span.end,
                    text: span.text.clone(),
                    score: span.score.clamp(0.0, 1.0),
                    recognizer_name: self.name.clone(),
                });
            }

            results
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
