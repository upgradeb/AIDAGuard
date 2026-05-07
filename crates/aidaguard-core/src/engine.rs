use std::path::Path;

use crate::detector::Match;

/// Abstract interface for a sensitive-data detection engine.
///
/// Implementations range from the built-in regex [`Detector`](crate::detector::Detector)
/// to the full [`AnalyzerEngine`] in `aidaguard-detector`.
pub trait DetectionEngine: Send + Sync {
    /// Scan `text` for sensitive data, returning all matches.
    fn detect(&self, text: &str) -> Vec<Match>;

    /// Number of currently loaded rules.
    fn rule_count(&self) -> usize;

    /// Look up a human-readable rule name by its ID.
    fn rule_name(&self, id: &str) -> Option<&str>;

    /// Reload rules from a directory of YAML files, replacing the current rule set.
    fn reload(&mut self, dir: &Path) -> Result<usize, anyhow::Error>;
}
