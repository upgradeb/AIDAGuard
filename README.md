# Aidaguard

> **The AI-native guardian between you and large language models.**
> Guard every token. Protect every prompt.

Aidaguard is an open-source desktop client that acts as a local API gateway proxy, automatically detecting, replacing, and restoring sensitive data before it reaches any LLM API — with zero configuration and no certificate installation required.

---

## How It Works

```
Your AI Client
      │
      │  HTTP  (localhost:7890)
      ▼
 Aidaguard Proxy  ←── detect & replace sensitive data
      │
      │  HTTPS (encrypted)
      ▼
 OpenAI / Any OpenAI-compatible API
      │
      ▼
 Aidaguard Proxy  ←── restore placeholders in response
      │
      ▼
Your AI Client
```

## Features

- 🛡️ **Local API Gateway** — No certificate required. Just point your API BaseURL to `http://localhost:7890`
- 🔍 **Smart Detection** — Built-in rules for phone numbers, ID cards, bank cards, emails, API keys, and more
- 🔄 **Seamless Replacement** — Sensitive data replaced with placeholders before sending; restored in responses
- 🌊 **Streaming Support** — Full SSE / stream mode support with sliding buffer restoration
- 📁 **File-based Rules** — YAML rule files, hot-reloadable, community-extendable
- 🔒 **Encrypted Storage** — Mapping tables stored locally with AES-256-GCM encryption
- 🖥️ **Cross-platform** — macOS, Windows, Linux via Tauri

## Quick Start

> Prerequisites: [Rust](https://rustup.rs) · [Node.js](https://nodejs.org) · [Tauri CLI](https://tauri.app)

```bash
git clone https://github.com/yourusername/aidaguard.git
cd aidaguard
cargo build
```

Then set your AI client's Base URL to:
```
http://localhost:7890
```

## Rule Files

Rules live in `~/.aidaguard/rules/` and are hot-reloaded on change.

Built-in rule packs:
- `general.yaml` — Phone, ID card, email, bank card, API keys
- `finance.yaml` — SWIFT codes, strict bank card patterns
- `medical.yaml` — Medical record numbers, diagnosis codes

## Project Structure

```
aidaguard/
├── crates/
│   ├── aidaguard-core/        # Core engine (proxy, detector, replacer, storage)
│   └── aidaguard-tauri/       # Desktop client (Tauri + React)
├── rules/                     # Built-in rule packs
└── docs/                      # Documentation
```

## License

MIT
