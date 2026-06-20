# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Project Overview

Aidaguard is a local LLM API gateway proxy that detects, replaces, and restores sensitive data (PII, credentials, etc.) before it reaches any LLM API. Built with Rust (backend) and Tauri + React (frontend).

## Common Commands

```bash
# Build the entire workspace
cargo build

# Build with NLP feature (BERT-based NER detection)
cargo build --features nlp

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p aidaguard-detector

# Run a specific test
cargo test test_detect_phone

# Run the Tauri desktop app (development mode)
cd crates/aidaguard-tauri/src-tauri && cargo tauri dev

# Check code without building
cargo check --all

# Format code
cargo fmt

# Lint
cargo clippy --all
```

## Architecture

7 crates organized in layers:

```
┌─────────────────────────────────────────────────────────────────┐
│  Application Layer: aidaguard-tauri (Tauri desktop app)         │
├─────────────────────────────────────────────────────────────────┤
│  Service Layer: proxy, upstream, plugins                        │
├─────────────────────────────────────────────────────────────────┤
│  Business Layer: aidaguard-detector (regex + checksum + NLP)    │
├─────────────────────────────────────────────────────────────────┤
│  Foundation Layer: aidaguard-core, aidaguard-storage            │
└─────────────────────────────────────────────────────────────────┘
```

### Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| `aidaguard-core` | Core types (`EntityType`, `Config`), traits (`DetectionEngine`), compiled YAML rule definitions, replacer |
| `aidaguard-detector` | Pattern recognizers with validators, NLP NER, `AnalyzerEngine` pipeline |
| `aidaguard-proxy` | HTTP reverse proxy (Axum), detection → replacement → forwarding → restoration |
| `aidaguard-storage` | SQLite audit logs with AES-256-GCM encryption |
| `aidaguard-upstream` | LLM provider definitions (OpenAI, Anthropic, DeepSeek, etc.) |
| `aidaguard-plugins` | AI tool adapters (Cursor, Cline, Codex, etc.) |
| `aidaguard-tauri` | Desktop app with Tauri commands and React frontend |

## Detection System (Unified Pipeline)

All detection goes through a single `RecognizerRegistry` pipeline.
YAML rules are converted to `YamlRecognizer` instances and loaded
into the same registry as the built-in pattern recognizers.

### Pattern Recognizers (`aidaguard-detector/src/recognizers/pattern/`)

Hardcoded Rust recognizers with:
- **Validators**: Luhn check (credit cards), mod-11 (Chinese ID), IBAN validation
- **Context words**: Confidence boosting when keywords appear nearby
- **Confidence scoring**: 0.0-1.0 range with overlap resolution

### YAML Rules (`aidaguard-core/src/detector/`)

Loaded from `rules/` directory (hot-reloadable), converted to `YamlRecognizer`
instances and registered into the recognizer pipeline. No separate legacy
`Detector` path — all detection is unified.

`RuleDef`: id, name, pattern, exclude, enabled, strategy, mode, priority, compliance

### AnalyzerEngine (`aidaguard-detector/src/pipeline.rs`)

```rust
// Unified pipeline: pattern recognizers + YAML rules in one pass
pub fn scan(&self, text: &str) -> Vec<RecognizerResult> {
    let mut results = self.registry.analyze_all(text);
    results = ConfidenceScorer::resolve_overlaps(results);
    results.retain(|r| r.score >= self.min_confidence);
    results
}
```

Both `detect()` and `detect_parallel()` go through the full pipeline
(overlap resolution + confidence filtering), guaranteeing consistent behavior.

## Recent Refactoring

- **Proxy server**: CORS middleware via `tower-http::CorsLayer` (Any origin/method/header)
- **SSE streaming**: Fixed to use line-by-line parsing instead of `split("\n\n")`,
  preventing JSON payload corruption on embedded blank lines
- **IncrementalRestorer**: Removed (dead code)
- **find_safe_len**: Now uses a precomputed prefix index instead of O(n\xc2\xb2) scanning
  on each streaming chunk
- **forward_headers**: Clones `HeaderValue` directly instead of re-parsing bytes
- **Settings page**: Layout aligned with Rules page pattern
  (`flex h-full flex-col` + `min-h-0 flex-1 overflow-auto`)

## Key Traits

### DetectionEngine (`aidaguard-core/src/engine.rs`)

```rust
pub trait DetectionEngine: Send + Sync {
    fn detect(&self, text: &str) -> Vec<Match>;
    fn rule_count(&self) -> usize;
    fn rule_name(&self, id: &str) -> Option<&str>;
    fn reload(&mut self, dir: &Path) -> Result<usize, DetectionError>;
    fn supported_entities(&self) -> Vec<EntityType>;
}
```

### Recognizer (`aidaguard-detector/src/core/recognizer.rs`)

```rust
pub trait Recognizer: Send + Sync {
    fn entity_type(&self) -> EntityType;
    fn name(&self) -> &str;
    fn analyze(&self, text: &str) -> Vec<RecognizerResult>;
    fn context_words(&self) -> &[String] { &[] }
}
```

## Rule Directory Structure

```
rules/
├── global/           # Always loaded as baseline
│   ├── credentials.yaml
│   ├── finance.yaml
│   └── identifiers.yaml
├── cn/               # China (PIPL)
├── eu/               # EU (GDPR)
├── gb/               # UK (DPA)
└── us/               # US (CCPA/HIPAA)
```

Rules are loaded via `Config::rule_presets()` which computes: `["global", region, region/industry, ...]`

## Tauri Commands

Located in `crates/aidaguard-tauri/src-tauri/src/commands/`:

| Module | Commands |
|--------|----------|
| `config.rs` | `get_config`, `save_config`, `get_app_version` |
| `proxy.rs` | `start_proxy`, `stop_proxy`, `proxy_status` |
| `rules.rs` | `get_rules`, `save_rule`, `delete_rule`, `test_rule`, `generate_rule` |
| `audit.rs` | `get_audit_records`, `get_audit_stats`, `export_csv`, `export_json` |
| `tools.rs` | `get_tools`, `configure_tool`, `restore_tool` |
| `upstream.rs` | `get_upstreams`, `add_upstream`, `delete_upstream`, `test_connectivity` |

## Frontend Stack

- React 18 + TypeScript
- shadcn/ui (Radix UI) + Tailwind CSS
- Zustand for state management
- Recharts for visualization
- React Hook Form + Zod for forms

## Adding a New Pattern Recognizer

1. Create `crates/aidaguard-detector/src/recognizers/pattern/my_entity.rs`:

```rust
use super::pattern_recognizer::PatternRecognizer;
use aidaguard_core::EntityType;
use regex::Regex;

pub fn new() -> PatternRecognizer {
    let pattern = Regex::new(r"your-pattern-here").expect("regex");
    PatternRecognizer::new(EntityType::Custom("my_entity".into()), "MyEntityRecognizer", pattern, 0.5)
        .with_validator(std::sync::Arc::new(|s: &str| /* validation logic */))
        .with_context_words(vec!["keyword1", "keyword2"])
}
```

2. Add module in `mod.rs`
3. Register in `RecognizerRegistry::load_predefined()`

## Validation Functions

Located in `aidaguard-detector/src/validation/`:

- `luhn.rs` - Credit card Luhn algorithm
- `id_card_cn.rs` - Chinese ID card (18-digit mod-11 checksum)
- `iban.rs` - IBAN structure validation
- `mod_n.rs` - Mod-N checksum utilities
- `context.rs` - Context word confidence enhancer

## Configuration (`config.toml`)

```toml
port = 19000
rules_dir = "./rules"
region = "cn"                    # cn, us, eu, gb, global
rule_industries = ["finance"]    # Sub-presets within region

[storage]
enabled = true
db_path = "./data/aidaguard.db"

[nlp]
enabled = false                  # Enable BERT NER (requires ~400MB model download)
default_language = "en"

[[upstreams]]
name = "openai"
url = "https://api.openai.com/v1"
api_key = "sk-xxx"
default = true
```

## When Making Changes

### Backward Compatibility

- YAML rule format must remain compatible - existing rules should work unchanged
- Tauri commands should maintain their signatures
- `DetectionEngine` trait is the stable interface

### Performance Considerations

- Large text detection: Use `scan_parallel()` for multi-core utilization
- NLP is CPU-intensive: Disabled by default, user opt-in required
- SQLite: Use WAL mode for concurrent reads/writes

### Adding New Entity Types

1. Add to `EntityType` enum in `aidaguard-core/src/entity.rs`
2. Update `EntityCategory::category()` mapping
3. Add `FromStr` implementation
4. Create corresponding pattern recognizer (if structured) or NLP mapping (if unstructured)
