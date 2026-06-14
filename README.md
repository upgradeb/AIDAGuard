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
 OpenAI / DeepSeek / Any OpenAI-compatible API
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

## Supported Sensitive Data Types

### China (PIPL)
- 居民身份证、护照、军官证、港澳通行证
- 手机号、车牌号、银行卡号
- 统一社会信用代码

### United States (CCPA/CPRA)
- Social Security Number (SSN)
- ITIN, EIN
- Driver License (state-specific)

### European Union (GDPR)
- IBAN, SWIFT/BIC
- VAT Numbers (country-specific)
- National ID Cards

### Other Regions
- 🇸🇬 Singapore: NRIC, FIN, UEN
- 🇯🇵 Japan: My Number
- 🇬🇧 UK: NINO, NHS Number
- 🇰🇷 Korea: Resident Registration Number

### Global
- Credit Cards (Visa, Mastercard, Amex, etc.) with Luhn validation
- Email addresses
- IP addresses (IPv4/IPv6)
- API Keys (OpenAI, AWS, GitHub, Stripe, etc.)
- JWT Tokens, Private Keys

## Quick Start

> Prerequisites: [Rust](https://rustup.rs) · [Node.js](https://nodejs.org) · [Tauri CLI](https://tauri.app)

```bash
git clone https://github.com/yourusername/aidaguard.git
cd aidaguard

# Build and run the desktop app
cd crates/aidaguard-tauri
cargo tauri dev
```

Then set your AI client's Base URL to:
```
http://localhost:7890
```

## Project Structure

```
aidaguard/
├── crates/
│   ├── aidaguard-core/        # Core types, config, detector, replacer
│   ├── aidaguard-detector/    # Detection engine (regex + checksum + NLP)
│   ├── aidaguard-proxy/       # HTTP proxy server (Axum)
│   ├── aidaguard-upstream/    # LLM provider management
│   ├── aidaguard-plugins/     # AI tool adapters (Cursor, Claude Code, etc.)
│   ├── aidaguard-storage/     # Encrypted audit storage (SQLite)
│   └── aidaguard-tauri/       # Desktop client (Tauri 2.x + React)
├── rules/                     # Built-in rule packs by region
└── docs/                      # Documentation
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System architecture and crate overview
- [Development Guide](docs/DEVELOPMENT.md) — Development setup and workflow
- [UI Design](docs/UI_DESIGN.md) — Desktop client UI design
- [Rules Optimization Plan](docs/RULES_OPTIMIZATION_PLAN.md) — Future improvements for detection rules

## Supported AI Tools

Aidaguard can automatically configure proxy settings for:

| Category | Tools |
|----------|-------|
| CLI | Claude Code, Aider, Codex CLI, Gemini CLI, OpenCode |
| IDE | Cursor, Zed, Windsurf, JetBrains AI |
| VS Code Extensions | Cline, Continue.dev, Codeium, Cody, Tabnine |

## Tech Stack

**Backend:**
- Rust, Tokio, Axum 0.7, Reqwest 0.12
- Candle (optional NLP/NER with BERT)

**Frontend:**
- Tauri 2.x, React 18, TypeScript
- shadcn/ui, Tailwind CSS, Zustand

## License

MIT
