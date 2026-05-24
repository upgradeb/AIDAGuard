mod roo_code;
mod cline;
mod continue_dev;
mod cursor;
mod windsurf;
mod zed;
mod aider;
mod claude_code;
mod openclaw;
mod hermes_agent;
mod codex;
mod gemini;
mod opencode;

// Phase 4: New tool adapters
mod copilot;
mod codeium;
mod cody;
mod tabnine;
mod codewhisperer;
mod jetbrains_ai;

pub use roo_code::RooCode;
pub use cline::Cline;
pub use continue_dev::ContinueDev;
pub use cursor::Cursor;
pub use windsurf::Windsurf;
pub use zed::Zed;
pub use aider::Aider;
pub use claude_code::ClaudeCode;
pub use openclaw::OpenClaw;
pub use hermes_agent::HermesAgent;
pub use codex::Codex;
pub use gemini::GeminiCli;
pub use opencode::OpenCode;

// Phase 4: New tool adapters
pub use copilot::Copilot;
pub use codeium::Codeium;
pub use cody::Cody;
pub use tabnine::Tabnine;
pub use codewhisperer::CodeWhisperer;
pub use jetbrains_ai::JetBrainsAI;

use crate::registry::PluginRegistry;

/// Register all built-in tool plugins into the registry.
pub fn register_all(registry: &mut PluginRegistry) {
    registry.register(Box::new(RooCode::new()));
    registry.register(Box::new(Cline::new()));
    registry.register(Box::new(ContinueDev::new()));
    registry.register(Box::new(Cursor::new()));
    registry.register(Box::new(Windsurf::new()));
    registry.register(Box::new(Zed::new()));
    registry.register(Box::new(Aider::new()));
    registry.register(Box::new(ClaudeCode::new()));
    registry.register(Box::new(OpenClaw::new()));
    registry.register(Box::new(HermesAgent::new()));
    registry.register(Box::new(Codex::new()));
    registry.register(Box::new(GeminiCli::new()));
    registry.register(Box::new(OpenCode::new()));

    // Phase 4: New tool adapters
    registry.register(Box::new(Copilot::new()));
    registry.register(Box::new(Codeium::new()));
    registry.register(Box::new(Cody::new()));
    registry.register(Box::new(Tabnine::new()));
    registry.register(Box::new(CodeWhisperer::new()));
    registry.register(Box::new(JetBrainsAI::new()));
}
