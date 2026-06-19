use aidaguard_core::detector::ValidatorFn;
use std::collections::HashMap;
use std::sync::Arc;

use super::iban;
use super::id_card_cn;
use super::luhn;

/// Registry that maps validator names to their validation functions.
/// Used to wire up YAML rule `validator` declarations to actual validation logic.
pub struct ValidatorRegistry {
    validators: HashMap<String, ValidatorFn>,
}

impl ValidatorRegistry {
    /// Create a new registry with all built-in validators registered.
    pub fn new() -> Self {
        let mut registry = Self {
            validators: HashMap::new(),
        };
        registry.register_builtin();
        registry
    }

    /// Register all built-in validators.
    fn register_builtin(&mut self) {
        self.register("luhn", Arc::new(luhn::luhn_check));
        self.register("id_card_cn", Arc::new(id_card_cn::validate_id_card_cn));
        self.register("iban", Arc::new(iban::validate_iban));
        self.register("us_ssn", Arc::new(validate_us_ssn));
    }

    /// Register a custom validator function by name.
    pub fn register(&mut self, name: &str, func: ValidatorFn) {
        self.validators.insert(name.to_string(), func);
    }

    /// Look up a validator by name.
    pub fn get(&self, name: &str) -> Option<&ValidatorFn> {
        self.validators.get(name)
    }

    /// Check if a validator with the given name exists.
    pub fn contains(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// List all registered validator names.
    pub fn names(&self) -> Vec<&String> {
        self.validators.keys().collect()
    }

    /// Wire up validators to a Detector's compiled rules.
    /// For each rule that declares a `validator` name, look up the function
    /// and assign it to the compiled rule's `validator_fn` field.
    pub fn apply_to_detector(&self, detector: &mut aidaguard_core::detector::Detector) {
        let pending = detector.rules_needing_validators();
        for (rule_id, validator_name) in pending {
            if let Some(validator_fn) = self.get(&validator_name) {
                detector.set_validator(&rule_id, validator_fn.clone());
            } else {
                tracing::warn!(
                    "规则 [{}] 声明了校验器 '{}'，但未在注册表中找到",
                    rule_id,
                    validator_name
                );
            }
        }
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// US SSN validation: area (001-899, != 666), group (01-99), serial (0001-9999).
fn validate_us_ssn(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let area: u32 = parts[0].parse().unwrap_or(0);
    let group: u32 = parts[1].parse().unwrap_or(0);
    let serial: u32 = parts[2].parse().unwrap_or(0);

    area != 0 && area != 666 && area < 900 && group != 0 && serial != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_builtins() {
        let registry = ValidatorRegistry::new();
        assert!(registry.contains("luhn"));
        assert!(registry.contains("id_card_cn"));
        assert!(registry.contains("iban"));
        assert!(registry.contains("us_ssn"));
    }

    #[test]
    fn test_registry_unknown_validator() {
        let registry = ValidatorRegistry::new();
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_luhn_via_registry() {
        let registry = ValidatorRegistry::new();
        let validator = registry.get("luhn").expect("luhn validator");
        assert!(validator("4532015112830366"));
        assert!(!validator("4532015112830367"));
    }

    #[test]
    fn test_id_card_cn_via_registry() {
        let registry = ValidatorRegistry::new();
        let validator = registry.get("id_card_cn").expect("id_card_cn validator");
        assert!(validator("110101199003076632"));
    }

    #[test]
    fn test_iban_via_registry() {
        let registry = ValidatorRegistry::new();
        let validator = registry.get("iban").expect("iban validator");
        assert!(validator("GB29NWBK60161331926819"));
    }

    #[test]
    fn test_us_ssn_via_registry() {
        let registry = ValidatorRegistry::new();
        let validator = registry.get("us_ssn").expect("us_ssn validator");
        assert!(validator("123-45-6789"));
        assert!(!validator("666-45-6789"));
        assert!(!validator("000-45-6789"));
    }

    #[test]
    fn test_custom_validator() {
        let mut registry = ValidatorRegistry::new();
        registry.register("always_true", Arc::new(|_s: &str| true));
        assert!(registry.contains("always_true"));
        let validator = registry.get("always_true").expect("custom validator");
        assert!(validator("anything"));
    }
}
