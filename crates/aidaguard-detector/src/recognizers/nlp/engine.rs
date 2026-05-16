use candle_core::{Module, Tensor};
use rayon::prelude::*;
use std::sync::Arc;

use crate::recognizers::nlp::registry::NlpModel;

/// BERT token limit - text longer than this requires chunking
const MAX_SEQ_LEN: usize = 512;
/// Overlap between chunks to avoid missing entities at boundaries
const CHUNK_OVERLAP: usize = 50;

#[derive(Debug, Clone)]
pub struct RawNerSpan {
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub score: f64,
}

pub struct NlpEngine {
    model: Arc<NlpModel>,
}

impl NlpEngine {
    pub fn new(model: Arc<NlpModel>) -> Self {
        Self { model }
    }

    /// Run NER inference on `text` and return raw entity spans with labels.
    ///
    /// For texts longer than BERT's max sequence length, automatically splits
    /// into overlapping chunks and merges results.
    pub fn run(&self, text: &str) -> Result<Vec<RawNerSpan>, anyhow::Error> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        // Estimate token count (roughly 4 chars per token on average)
        let estimated_tokens = text.len() / 4;

        if estimated_tokens <= MAX_SEQ_LEN - CHUNK_OVERLAP {
            // Short text: single inference
            self.run_single(text)
        } else {
            // Long text: chunked parallel inference
            self.run_chunked(text)
        }
    }

    /// Single-pass inference for short texts.
    fn run_single(&self, text: &str) -> Result<Vec<RawNerSpan>, anyhow::Error> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let encoding = self
            .model
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        let seq_len = encoding.len().min(self.model.config.max_position_embeddings);
        let ids: Vec<u32> = encoding.get_ids()[..seq_len].to_vec();
        let mask: Vec<u32> = encoding.get_attention_mask()[..seq_len].to_vec();
        let type_ids: Vec<u32> = encoding.get_type_ids().iter().take(seq_len).copied().collect();

        let input_ids = Tensor::from_slice(ids.as_slice(), (1, seq_len), &self.model.device)?;
        let token_type_ids =
            Tensor::from_slice(type_ids.as_slice(), (1, seq_len), &self.model.device)?;
        let attention_mask =
            Tensor::from_slice(mask.as_slice(), (1, seq_len), &self.model.device)?;

        // Run BERT encoder
        let hidden_states = self.model.bert.forward(
            &input_ids,
            &token_type_ids,
            Some(&attention_mask),
        )?;

        // Classification head
        let logits = self.model.classifier.forward(&hidden_states)?;

        // softmax over last dim + argmax
        let probs = candle_nn::ops::softmax(&logits, 2)?;
        let predicted_ids = probs.argmax(2)?;
        let predicted_ids: Vec<u32> = predicted_ids.squeeze(0)?.to_vec1()?;

        // Gather confidence scores
        let probs_squeezed: Vec<Vec<f32>> = probs.squeeze(0)?.to_vec2()?;
        let probs_per_token: Vec<f32> = predicted_ids
            .iter()
            .enumerate()
            .map(|(i, &id)| {
                probs_squeezed
                    .get(i)
                    .and_then(|row: &Vec<f32>| row.get(id as usize).copied())
                    .unwrap_or(0.0)
            })
            .collect();

        // Convert label IDs to label strings
        let labels: Vec<&str> = predicted_ids
            .iter()
            .map(|&id| {
                self.model
                    .id_to_label
                    .get(id as usize)
                    .map(|s| s.as_str())
                    .unwrap_or("O")
            })
            .collect();

        // Convert offsets
        let raw_offsets: Vec<(usize, usize)> = encoding.get_offsets().to_vec();
        let offsets: Vec<Option<(usize, usize)>> = raw_offsets
            .into_iter()
            .map(|(s, e)| if s == 0 && e == 0 { None } else { Some((s, e)) })
            .collect();

        let spans = decode_bio_spans(text, &labels, &probs_per_token, &offsets, seq_len);
        Ok(spans)
    }

    /// Chunked parallel inference for long texts.
    ///
    /// Splits text into overlapping chunks, runs inference in parallel,
    /// then merges and deduplicates results.
    fn run_chunked(&self, text: &str) -> Result<Vec<RawNerSpan>, anyhow::Error> {
        let chunks = self.split_into_chunks(text);

        // Parallel inference on each chunk
        let chunk_results: Vec<Vec<RawNerSpan>> = chunks
            .par_iter()
            .filter_map(|(chunk, offset)| {
                self.run_single(chunk).ok().map(|spans| {
                    // Adjust offsets to original text position
                    spans
                        .into_iter()
                        .map(|mut span| {
                            span.start += offset;
                            span.end += offset;
                            span
                        })
                        .collect()
                })
            })
            .collect();

        // Merge and deduplicate
        let mut all_spans: Vec<RawNerSpan> = chunk_results.into_iter().flatten().collect();

        // Sort by start position
        all_spans.sort_by(|a, b| a.start.cmp(&b.start));

        // Deduplicate overlapping spans (keep higher score)
        let mut deduped = Vec::new();
        for span in all_spans {
            let overlaps = deduped.iter().any(|existing| {
                span.start < existing.end && span.end > existing.start
            });

            if overlaps {
                // Find overlapping span and keep higher score
                if let Some(existing) = deduped.iter_mut().find(|e| {
                    span.start < e.end && span.end > e.start && span.score > e.score
                }) {
                    *existing = span;
                }
            } else {
                deduped.push(span);
            }
        }

        Ok(deduped)
    }

    /// Split text into overlapping chunks for processing.
    /// Returns (chunk_text, offset_in_original) pairs.
    fn split_into_chunks(&self, text: &str) -> Vec<(String, usize)> {
        let mut chunks = Vec::new();
        let chunk_chars = MAX_SEQ_LEN * 4; // Approximate chars per chunk
        let overlap_chars = CHUNK_OVERLAP * 4;

        let mut start = 0;
        let text_len = text.len();

        while start < text_len {
            let end = (start + chunk_chars).min(text_len);

            // Find a good break point (space or punctuation)
            let break_point = if end < text_len {
                text[start..end]
                    .rfind(|c: char| c.is_whitespace() || c == '.' || c == ',' || c == '!')
                    .map(|i| start + i + 1)
                    .unwrap_or(end)
            } else {
                end
            };

            let chunk = text[start..break_point].to_string();
            if !chunk.is_empty() {
                chunks.push((chunk, start));
            }

            start = if break_point > overlap_chars {
                break_point - overlap_chars
            } else {
                break_point
            };
        }

        chunks
    }
}

/// Decode BIO tag sequence into contiguous entity spans.
fn decode_bio_spans(
    text: &str,
    labels: &[&str],
    scores: &[f32],
    offsets: &[Option<(usize, usize)>],
    seq_len: usize,
) -> Vec<RawNerSpan> {
    let mut spans = Vec::new();
    let mut current_label: Option<&str> = None;
    let mut span_start: Option<usize> = None;
    let mut score_sum: f64 = 0.0;
    let mut score_count: usize = 0;

    for i in 0..seq_len {
        let label = labels[i];
        let (prefix, entity) = split_bio(label);

        match prefix {
            "B" => {
                flush_span(
                    &mut spans, text, offsets,
                    current_label, span_start, i, score_sum, score_count,
                );
                current_label = entity;
                span_start = offsets.get(i).and_then(|o| *o).map(|o| o.0);
                score_sum = *scores.get(i).unwrap_or(&0.0) as f64;
                score_count = 1;
            }
            "I" => {
                if current_label == entity {
                    score_sum += *scores.get(i).unwrap_or(&0.0) as f64;
                    score_count += 1;
                } else {
                    flush_span(
                        &mut spans, text, offsets,
                        current_label, span_start, i, score_sum, score_count,
                    );
                    current_label = entity;
                    span_start = offsets.get(i).and_then(|o| *o).map(|o| o.0);
                    score_sum = *scores.get(i).unwrap_or(&0.0) as f64;
                    score_count = 1;
                }
            }
            _ => {
                flush_span(
                    &mut spans, text, offsets,
                    current_label, span_start, i, score_sum, score_count,
                );
                current_label = None;
                span_start = None;
                score_sum = 0.0;
                score_count = 0;
            }
        }
    }

    flush_span(
        &mut spans, text, offsets,
        current_label, span_start, seq_len, score_sum, score_count,
    );

    spans
}

fn flush_span(
    spans: &mut Vec<RawNerSpan>,
    text: &str,
    offsets: &[Option<(usize, usize)>],
    label: Option<&str>,
    start: Option<usize>,
    token_idx: usize,
    score_sum: f64,
    score_count: usize,
) {
    if let (Some(lbl), Some(st)) = (label, start) {
        let end = find_char_end(text, offsets, token_idx);
        if end > st {
            let avg_score = score_sum / score_count.max(1) as f64;
            spans.push(RawNerSpan {
                label: lbl.to_string(),
                start: st,
                end,
                text: text[st..end].to_string(),
                score: avg_score,
            });
        }
    }
}

fn split_bio(label: &str) -> (&str, Option<&str>) {
    if let Some(rest) = label.strip_prefix("B-") {
        ("B", Some(rest))
    } else if let Some(rest) = label.strip_prefix("I-") {
        ("I", Some(rest))
    } else {
        ("O", None)
    }
}

fn find_char_end(text: &str, offsets: &[Option<(usize, usize)>], token_idx: usize) -> usize {
    for i in (0..token_idx.min(offsets.len())).rev() {
        if let Some((_, end)) = offsets[i] {
            if end > 0 {
                return end.min(text.len());
            }
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_bio_parses_correctly() {
        assert_eq!(split_bio("B-PER"), ("B", Some("PER")));
        assert_eq!(split_bio("I-LOC"), ("I", Some("LOC")));
        assert_eq!(split_bio("O"), ("O", None));
    }
}
