use crate::core::result::RecognizerResult;

/// Strategies for computing and adjusting confidence scores.
pub struct ConfidenceScorer;

impl ConfidenceScorer {
    /// Adjust confidence based on presence of context words near the match span.
    /// Each context word found in the surrounding window adds `boost` to the score.
    pub fn enhance_with_context(
        result: &RecognizerResult,
        text: &str,
        context_words: &[String],
        context_radius: usize,
        boost_per_word: f64,
    ) -> f64 {
        if context_words.is_empty() {
            return result.score;
        }

        let window_start = result.start.saturating_sub(context_radius);
        let window_end = (result.end + context_radius).min(text.len());

        // Ensure we're on valid UTF-8 boundaries
        let mut s = window_start;
        while s > 0 && !text.is_char_boundary(s) {
            s -= 1;
        }
        let mut e = window_end;
        while e < text.len() && !text.is_char_boundary(e) {
            e += 1;
        }

        let window = &text[s..e].to_lowercase();
        let mut bonus = 0.0;

        for word in context_words {
            if window.contains(&word.to_lowercase()) {
                bonus += boost_per_word;
            }
        }

        (result.score + bonus).min(1.0)
    }

    /// Resolve overlapping results by keeping the highest-confidence match per span.
    /// Results are sorted by start position, then by confidence (descending).
    pub fn resolve_overlaps(results: Vec<RecognizerResult>) -> Vec<RecognizerResult> {
        if results.is_empty() {
            return results;
        }

        // Sort by start ascending, then by score descending
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
        });

        let mut resolved: Vec<RecognizerResult> = Vec::new();
        for r in sorted {
            // Deduplicate: same start, end, and entity_type
            if resolved.iter().any(|existing| {
                existing.entity_type == r.entity_type
                    && existing.start == r.start
                    && existing.end == r.end
            }) {
                continue;
            }
            // Remove overlap: if this result overlaps with any already-selected result,
            // keep the one with higher confidence
            if let Some(pos) = resolved.iter().position(|existing| {
                r.start < existing.end && r.end > existing.start
            }) {
                if r.score > resolved[pos].score {
                    resolved[pos] = r;
                }
            } else {
                resolved.push(r);
            }
        }

        resolved
    }

    /// Clamp a score to [0.0, 1.0].
    pub fn normalize(score: f64) -> f64 {
        score.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidaguard_core::EntityType;

    #[test]
    fn test_enhance_with_context_no_words() {
        let result = RecognizerResult {
            entity_type: EntityType::CreditCard,
            start: 0,
            end: 16,
            text: "4532015112830366".into(),
            score: 0.5,
            recognizer_name: "test".into(),
        };
        let score = ConfidenceScorer::enhance_with_context(
            &result,
            "card 4532015112830366",
            &[],
            20,
            0.15,
        );
        assert_eq!(score, 0.5);
    }

    #[test]
    fn test_enhance_with_context_found() {
        let result = RecognizerResult {
            entity_type: EntityType::CreditCard,
            start: 5,
            end: 21,
            text: "4532015112830366".into(),
            score: 0.5,
            recognizer_name: "test".into(),
        };
        let score = ConfidenceScorer::enhance_with_context(
            &result,
            "card 4532015112830366",
            &["card".into()],
            20,
            0.15,
        );
        assert!(score > 0.5);
    }

    #[test]
    fn test_resolve_overlaps_prefer_higher_score() {
        let r1 = RecognizerResult {
            entity_type: EntityType::CreditCard,
            start: 0,
            end: 16,
            text: "4532015112830366".into(),
            score: 0.5,
            recognizer_name: "low".into(),
        };
        let r2 = RecognizerResult {
            entity_type: EntityType::CreditCard,
            start: 0,
            end: 16,
            text: "4532015112830366".into(),
            score: 0.9,
            recognizer_name: "high".into(),
        };
        let resolved = ConfidenceScorer::resolve_overlaps(vec![r1.clone(), r2.clone()]);
        // Same start/end/entity_type → deduplicated, first one wins (after sort by score desc)
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn test_normalize_clamps() {
        assert_eq!(ConfidenceScorer::normalize(1.5), 1.0);
        assert_eq!(ConfidenceScorer::normalize(-0.5), 0.0);
        assert_eq!(ConfidenceScorer::normalize(0.75), 0.75);
    }
}
