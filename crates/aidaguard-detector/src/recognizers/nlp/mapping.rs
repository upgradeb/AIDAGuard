use aidaguard_core::EntityType;
use std::collections::HashMap;

/// Maps standard NER token classification labels (B-PER, I-LOC, etc.) to
/// Aidaguard [`EntityType`] variants.
///
/// In Phase 1, only PER/LOC/ORG labels are mapped. MISC and unmapped labels
/// are returned as `None`.
pub struct LabelMapping {
    map: HashMap<String, EntityType>,
}

impl LabelMapping {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("B-PER".to_string(), EntityType::PersonName);
        map.insert("I-PER".to_string(), EntityType::PersonName);
        map.insert("B-LOC".to_string(), EntityType::Address);
        map.insert("I-LOC".to_string(), EntityType::Address);
        map.insert("B-ORG".to_string(), EntityType::Organization);
        map.insert("I-ORG".to_string(), EntityType::Organization);
        Self { map }
    }

    pub fn entity_type_for(&self, label: &str) -> Option<EntityType> {
        self.map.get(label).cloned()
    }

    pub fn is_relevant(&self, label: &str, target: &EntityType) -> bool {
        self.entity_type_for(label).as_ref() == Some(target)
    }
}

impl Default for LabelMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_per_to_person_name() {
        let m = LabelMapping::new();
        assert_eq!(m.entity_type_for("B-PER"), Some(EntityType::PersonName));
        assert_eq!(m.entity_type_for("I-PER"), Some(EntityType::PersonName));
    }

    #[test]
    fn maps_loc_to_address() {
        let m = LabelMapping::new();
        assert_eq!(m.entity_type_for("B-LOC"), Some(EntityType::Address));
        assert_eq!(m.entity_type_for("I-LOC"), Some(EntityType::Address));
    }

    #[test]
    fn maps_org_to_organization() {
        let m = LabelMapping::new();
        assert_eq!(m.entity_type_for("B-ORG"), Some(EntityType::Organization));
        assert_eq!(m.entity_type_for("I-ORG"), Some(EntityType::Organization));
    }

    #[test]
    fn returns_none_for_unknown_label() {
        let m = LabelMapping::new();
        assert_eq!(m.entity_type_for("B-MISC"), None);
        assert_eq!(m.entity_type_for("O"), None);
    }

    #[test]
    fn is_relevant_filters_correctly() {
        let m = LabelMapping::new();
        assert!(m.is_relevant("B-PER", &EntityType::PersonName));
        assert!(!m.is_relevant("B-LOC", &EntityType::PersonName));
    }
}
