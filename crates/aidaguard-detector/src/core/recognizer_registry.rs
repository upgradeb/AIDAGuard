use std::collections::HashMap;
use std::sync::Arc;

use crate::core::recognizer::Recognizer;
use crate::core::result::RecognizerResult;

/// Registry of all installed recognizers. Supports preloading pattern recognizers
/// and on-demand loading of NLP models.
pub struct RecognizerRegistry {
    recognizers: Vec<Arc<dyn Recognizer>>,
    /// Entity type display name → recognizer name, for DetectionEngine::rule_name()
    entity_names: HashMap<String, String>,
}

impl RecognizerRegistry {
    pub fn new() -> Self {
        Self {
            recognizers: Vec::new(),
            entity_names: HashMap::new(),
        }
    }

    /// Register a recognizer. Returns self for chaining.
    pub fn register(&mut self, recognizer: Arc<dyn Recognizer>) -> &mut Self {
        let key = recognizer.entity_type().to_string();
        let name = recognizer.name().to_string();
        self.entity_names.entry(key).or_insert(name);
        self.recognizers.push(recognizer);
        self
    }

    /// Load all built-in pattern recognizers. Called at engine initialization.
    pub fn load_predefined(&mut self) -> &mut Self {
        use crate::recognizers::pattern;

        self.register(Arc::new(pattern::credit_card::new()));
        self.register(Arc::new(pattern::id_card_cn::new()));
        self.register(Arc::new(pattern::passport_cn::new()));
        self.register(Arc::new(pattern::us_ssn::new()));
        self.register(Arc::new(pattern::uk_nino::new()));
        self.register(Arc::new(pattern::iban::new()));
        self.register(Arc::new(pattern::swift_code::new()));
        self.register(Arc::new(pattern::phone::new()));
        self.register(Arc::new(pattern::car_plate::new()));
        self.register(Arc::new(pattern::bank_account::new()));
        self.register(Arc::new(pattern::amount::new()));
        self.register(Arc::new(pattern::crypto_address::new()));
        self.register(Arc::new(pattern::email::new()));
        self.register(Arc::new(pattern::ip_address::new()));
        self.register(Arc::new(pattern::mac_address::new()));
        self.register(Arc::new(pattern::url::new()));
        self.register(Arc::new(pattern::api_key::new()));
        self.register(Arc::new(pattern::jwt::new()));
        self.register(Arc::new(pattern::aws_access_key::new()));
        self.register(Arc::new(pattern::private_key::new()));

        self
    }

    /// Load NLP recognizers for all unstructured entity types.
    /// `language` selects the NER model (e.g. "en", "zh").
    /// When the `nlp` feature is disabled, this is a no-op.
    pub fn load_nlp_recognizers(&mut self, language: &str) -> &mut Self {
        #[cfg(feature = "nlp")]
        {
            use crate::recognizers::nlp::NlpRecognizer;
            use aidaguard_core::EntityType;
            let lang = language.to_string();
            self.register(Arc::new(NlpRecognizer::new(EntityType::PersonName, "PersonNameNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Address, "AddressNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Organization, "OrganizationNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::DateOfBirth, "DateOfBirthNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Age, "AgeNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Nationality, "NationalityNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Religion, "ReligionNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::MedicalTerm, "MedicalTermNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Occupation, "OccupationNlpRecognizer").with_language(&lang)));
            self.register(Arc::new(NlpRecognizer::new(EntityType::Education, "EducationNlpRecognizer").with_language(&lang)));
        }
        #[cfg(not(feature = "nlp"))]
        {
            let _ = language;
        }
        self
    }

    /// Run all registered recognizers against text and collect all results.
    pub fn analyze_all(&self, text: &str) -> Vec<RecognizerResult> {
        let mut results = Vec::new();
        for recognizer in &self.recognizers {
            results.extend(recognizer.analyze(text));
        }
        results
    }

    /// Number of registered recognizers.
    pub fn recognizer_count(&self) -> usize {
        self.recognizers.len()
    }

    /// Look up the display name for an entity type id (e.g. "CREDIT_CARD" → "CreditCardRecognizer").
    pub fn entity_name(&self, id: &str) -> Option<&str> {
        self.entity_names.get(id).map(|s| s.as_str())
    }

    /// Iterate over all recognizers.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Recognizer>> {
        self.recognizers.iter()
    }
}

impl Default for RecognizerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
