// Complex adapters that require custom Rust code
// (YAML/TOML/XML configs, env files, AWS credentials, multi-IDE scanning)
mod aider;
mod codex;
mod hermes_agent;
mod gemini;
mod codewhisperer;
mod jetbrains_ai;

pub use aider::Aider;
pub use codex::Codex;
pub use hermes_agent::HermesAgent;
pub use gemini::GeminiCli;
pub use codewhisperer::CodeWhisperer;
pub use jetbrains_ai::JetBrainsAI;

use crate::registry::PluginRegistry;
use crate::declarative::loader;

/// Register all built-in tool plugins into the registry.
///
/// Declarative adapters (loaded from compile-time JSON manifests):
///   cursor, cline, claude_code, zed, continue, codeium, cody, tabnine,
///   openclaw, opencode, windsurf, qwencode, coffeecli, grok, openfang,
///   pi, picoclaw, nanobot, zeroclaw, claudedesktop, codexdesktop,
///   geminidesktop, vscode, trae, traecn
///
/// Complex adapters (custom Rust code for non-JSON configs):
///   aider, codex, hermes, gemini, codewhisperer, jetbrains
pub fn register_all(registry: &mut PluginRegistry) {
    // Load all 25 declarative adapters from compile-time embedded manifests
    for plugin in loader::load_all() {
        registry.register(plugin);
    }

    // Register complex adapters with custom configuration logic
    registry.register(Box::new(Aider::new()));
    registry.register(Box::new(Codex::new()));
    registry.register(Box::new(HermesAgent::new()));
    registry.register(Box::new(GeminiCli::new()));
    registry.register(Box::new(CodeWhisperer::new()));
    registry.register(Box::new(JetBrainsAI::new()));
}
