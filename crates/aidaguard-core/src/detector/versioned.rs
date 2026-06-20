//! Rule version management with snapshot and rollback support.

use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::CompiledRule;

/// A snapshot of compiled rules at a point in time.
/// Enables atomic switching and rollback.
#[derive(Debug, Clone)]
pub struct RuleSnapshot {
    /// Monotonically increasing version number
    pub version: u64,
    /// Timestamp when snapshot was created (milliseconds since Unix epoch)
    pub timestamp_ms: i64,
    /// The compiled rules
    pub rules: Vec<CompiledRule>,
    /// SHA-256 checksum of rule definitions for integrity verification
    pub checksum: String,
}

impl RuleSnapshot {
    /// Create a new snapshot from compiled rules.
    pub fn new(version: u64, rules: Vec<CompiledRule>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let checksum = compute_checksum(&rules);

        Self {
            version,
            timestamp_ms,
            rules,
            checksum,
        }
    }

    /// Number of rules in this snapshot.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// Compute SHA-256 checksum of rule definitions.
fn compute_checksum(rules: &[CompiledRule]) -> String {
    let mut hasher = Sha256::new();

    for rule in rules {
        // Hash rule ID, pattern, and key properties
        hasher.update(rule.def.id.as_bytes());
        hasher.update(rule.def.pattern.as_bytes());
        hasher.update(&[rule.def.enabled as u8]);
        let priority_bytes = rule.def.priority.to_le_bytes();
        hasher.update(&priority_bytes);
    }

    format!("{:x}", hasher.finalize())
}

/// A versioned detector with snapshot history for rollback support.
///
/// Maintains a current snapshot and a history of recent snapshots,
/// enabling atomic rule updates and quick rollback on failure.
pub struct VersionedDetector {
    /// Current active snapshot
    current: Arc<RuleSnapshot>,
    /// History of recent snapshots (for rollback)
    history: VecDeque<Arc<RuleSnapshot>>,
    /// Maximum number of historical snapshots to retain
    max_history: usize,
}

impl VersionedDetector {
    /// Create a new versioned detector with empty rules.
    pub fn new(max_history: usize) -> Self {
        let initial = RuleSnapshot::new(0, Vec::new());
        Self {
            current: Arc::new(initial),
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Create from existing rules.
    pub fn from_rules(rules: Vec<CompiledRule>, max_history: usize) -> Self {
        let initial = RuleSnapshot::new(0, rules);
        Self {
            current: Arc::new(initial),
            history: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Get the current snapshot.
    pub fn current(&self) -> &Arc<RuleSnapshot> {
        &self.current
    }

    /// Get current version number.
    pub fn version(&self) -> u64 {
        self.current.version
    }

    /// Get current rule count.
    pub fn rule_count(&self) -> usize {
        self.current.rules.len()
    }

    /// Atomically switch to new rules.
    ///
    /// 1. Validates new rules (non-empty)
    /// 2. Creates a new snapshot
    /// 3. Saves current to history
    /// 4. Atomically swaps to new snapshot
    ///
    /// Returns the new version number.
    pub fn atomic_swap(&mut self, new_rules: Vec<CompiledRule>) -> Result<u64, VersionError> {
        // Validate: allow empty rules but warn
        if new_rules.is_empty() {
            tracing::warn!("Switching to empty rule set");
        }

        // Create new snapshot
        let new_version = self.current.version + 1;
        let new_snapshot = Arc::new(RuleSnapshot::new(new_version, new_rules));

        // Save current to history
        self.history.push_back(self.current.clone());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Atomic swap
        self.current = new_snapshot;

        tracing::info!(
            "Rule snapshot v{}: {} rules, checksum={}",
            self.current.version,
            self.current.rules.len(),
            &self.current.checksum[..8]
        );

        Ok(self.current.version)
    }

    /// Rollback to the previous version.
    ///
    /// Returns the version number after rollback, or error if no history.
    pub fn rollback(&mut self) -> Result<u64, VersionError> {
        if let Some(prev) = self.history.pop_back() {
            self.current = prev;
            tracing::info!(
                "Rolled back to v{}: {} rules",
                self.current.version,
                self.current.rules.len()
            );
            Ok(self.current.version)
        } else {
            Err(VersionError::NoHistory)
        }
    }

    /// Rollback to a specific version.
    ///
    /// Searches history for the specified version and restores it.
    pub fn rollback_to(&mut self, version: u64) -> Result<u64, VersionError> {
        let pos = self
            .history
            .iter()
            .position(|s| s.version == version)
            .ok_or(VersionError::VersionNotFound { version })?;

        // Remove all snapshots after the target
        for _ in 0..=(self.history.len() - 1 - pos) {
            if let Some(s) = self.history.pop_back() {
                if s.version == version {
                    self.current = Arc::new((*s).clone());
                    tracing::info!("Rolled back to v{}", version);
                    return Ok(version);
                }
            }
        }

        Err(VersionError::VersionNotFound { version })
    }

    /// List available versions in history.
    pub fn history_versions(&self) -> Vec<(u64, i64, usize)> {
        self.history
            .iter()
            .map(|s| (s.version, s.timestamp_ms, s.rules.len()))
            .collect()
    }

    /// Clear history (free memory).
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for VersionedDetector {
    fn default() -> Self {
        Self::new(10)
    }
}

/// Errors for version management operations.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("No history available for rollback")]
    NoHistory,

    #[error("Version {version} not found in history")]
    VersionNotFound { version: u64 },

    #[error("Rule validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::{Mode, RuleDef, Strategy};
    use regex::Regex;

    fn make_test_rule(id: &str, pattern: &str) -> CompiledRule {
        CompiledRule {
            def: RuleDef {
                id: id.to_string(),
                name: format!("Test rule {}", id),
                pattern: pattern.to_string(),
                exclude: None,
                enabled: true,
                strategy: Strategy::Placeholder,
                mode: Mode::Filter,
                priority: 100,
                compliance: Vec::new(),
                validator: None,
                context_words: Vec::new(),
                base_confidence: None,
                region: None,
                source: "system".to_string(),
            },
            regex: Regex::new(pattern).unwrap(),
            exclude_regex: None,
            validator_fn: None,
        }
    }

    #[test]
    fn test_versioned_detector_basic() {
        let mut detector = VersionedDetector::new(5);

        assert_eq!(detector.version(), 0);
        assert_eq!(detector.rule_count(), 0);

        // Add rules
        let rules = vec![
            make_test_rule("email", r"\b[\w.-]+@[\w.-]+\.\w+\b"),
            make_test_rule("phone", r"\b\d{11}\b"),
        ];

        let v = detector.atomic_swap(rules).unwrap();
        assert_eq!(v, 1);
        assert_eq!(detector.rule_count(), 2);
    }

    #[test]
    fn test_rollback() {
        let mut detector = VersionedDetector::new(5);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];

        detector.atomic_swap(rules_v1).unwrap();
        detector.atomic_swap(rules_v2).unwrap();

        assert_eq!(detector.version(), 2);
        assert_eq!(detector.current().rules[0].def.id, "v2");

        // Rollback
        let v = detector.rollback().unwrap();
        assert_eq!(v, 1);
        assert_eq!(detector.current().rules[0].def.id, "v1");
    }

    #[test]
    fn test_checksum_consistency() {
        let rules = vec![
            make_test_rule("email", r"\b[\w.-]+@[\w.-]+\.\w+\b"),
            make_test_rule("phone", r"\b\d{11}\b"),
        ];

        let snap1 = RuleSnapshot::new(1, rules.clone());
        let snap2 = RuleSnapshot::new(2, rules);

        // Same rules should produce same checksum
        assert_eq!(snap1.checksum, snap2.checksum);
    }

    #[test]
    fn test_from_rules() {
        let rule1 = make_test_rule("r1", r"\d+");
        let rule2 = make_test_rule("r2", r"\w+");
        let detector = VersionedDetector::from_rules(vec![rule1, rule2], 5);

        assert_eq!(detector.version(), 0);
        assert_eq!(detector.rule_count(), 2);
    }

    #[test]
    fn test_rollback_to_specific_version() {
        let mut detector = VersionedDetector::new(10);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];
        let rules_v3 = vec![make_test_rule("v3", r"[a-z]+")];

        detector.atomic_swap(rules_v1).unwrap(); // v1 in history, current=v1
        detector.atomic_swap(rules_v2).unwrap(); // v2 in history, current=v2
        detector.atomic_swap(rules_v3).unwrap(); // v3 in history, current=v3

        assert_eq!(detector.version(), 3);

        let v = detector.rollback_to(1).unwrap();
        assert_eq!(v, 1);
        assert_eq!(detector.version(), 1);
    }

    #[test]
    fn test_rollback_to_nonexistent_version() {
        let mut detector = VersionedDetector::new(10);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];

        detector.atomic_swap(rules_v1).unwrap();
        detector.atomic_swap(rules_v2).unwrap();

        let result = detector.rollback_to(99);
        assert!(matches!(result, Err(VersionError::VersionNotFound { version: 99 })));
    }

    #[test]
    fn test_history_versions() {
        let mut detector = VersionedDetector::new(10);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];
        let rules_v3 = vec![make_test_rule("v3", r"[a-z]+")];

        detector.atomic_swap(rules_v1).unwrap();
        detector.atomic_swap(rules_v2).unwrap();
        detector.atomic_swap(rules_v3).unwrap();

        let versions = detector.history_versions();
        assert_eq!(versions.len(), 3);
        // History: v0 (initial empty, 0 rules), v1 (1 rule), v2 (1 rule)
        assert_eq!(versions[0].0, 0);
        assert_eq!(versions[0].2, 0);
        assert_eq!(versions[1].0, 1);
        assert_eq!(versions[1].2, 1);
        assert_eq!(versions[2].0, 2);
        assert_eq!(versions[2].2, 1);
    }

    #[test]
    fn test_clear_history() {
        let mut detector = VersionedDetector::new(10);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];

        detector.atomic_swap(rules_v1).unwrap();
        detector.atomic_swap(rules_v2).unwrap();

        assert!(!detector.history_versions().is_empty());

        detector.clear_history();

        let result = detector.rollback();
        assert!(matches!(result, Err(VersionError::NoHistory)));
    }

    #[test]
    fn test_rule_snapshot_len() {
        let rules = vec![
            make_test_rule("r1", r"\d+"),
            make_test_rule("r2", r"\w+"),
            make_test_rule("r3", r"[a-z]+"),
        ];
        let snapshot = RuleSnapshot::new(1, rules);
        assert_eq!(snapshot.len(), 3);
    }

    #[test]
    fn test_rule_snapshot_is_empty() {
        let empty = RuleSnapshot::new(1, Vec::new());
        assert!(empty.is_empty());

        let non_empty = RuleSnapshot::new(2, vec![make_test_rule("r1", r"\d+")]);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_max_history_respected() {
        let mut detector = VersionedDetector::new(2);

        for i in 0..5 {
            let rules = vec![make_test_rule(&format!("v{}", i), r"\d+")];
            detector.atomic_swap(rules).unwrap();
        }

        let versions = detector.history_versions();
        assert!(versions.len() <= 2);
    }

    #[test]
    fn test_rollback_after_clear_prevented() {
        let mut detector = VersionedDetector::new(10);

        let rules_v1 = vec![make_test_rule("v1", r"\d+")];
        let rules_v2 = vec![make_test_rule("v2", r"\w+")];

        detector.atomic_swap(rules_v1).unwrap();
        detector.atomic_swap(rules_v2).unwrap();

        detector.clear_history();

        let result = detector.rollback();
        assert!(matches!(result, Err(VersionError::NoHistory)));
    }
}
