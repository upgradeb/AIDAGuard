use crate::Plugin;
use super::engine::DeclarativeAdapter;
use super::manifest::ToolManifest;

/// Load a manifest from embedded JSON and create a DeclarativeAdapter.
fn load_manifest(json_str: &str) -> Result<DeclarativeAdapter, String> {
    let manifest: ToolManifest = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;
    Ok(DeclarativeAdapter::new(manifest))
}

/// Load all declarative tool adapters from compile-time embedded manifests.
/// New tools can be added by:
/// 1. Creating a manifest JSON in `crates/aidaguard-plugins/manifests/`
/// 2. Adding an `include_str!` line below
/// 3. Recompiling
pub fn load_all() -> Vec<Box<dyn Plugin>> {
    let manifests: &[&str] = &[
        include_str!("../../manifests/cursor.json"),
        include_str!("../../manifests/cline.json"),
        include_str!("../../manifests/claude_code.json"),
        include_str!("../../manifests/zed.json"),
        include_str!("../../manifests/continue.json"),
        include_str!("../../manifests/codeium.json"),
        include_str!("../../manifests/cody.json"),
        include_str!("../../manifests/tabnine.json"),
        include_str!("../../manifests/openclaw.json"),
        include_str!("../../manifests/opencode.json"),
        include_str!("../../manifests/windsurf.json"),
        // EchoBird-aligned tools
        include_str!("../../manifests/qwencode.json"),
        include_str!("../../manifests/coffeecli.json"),
        include_str!("../../manifests/grok.json"),
        include_str!("../../manifests/openfang.json"),
        include_str!("../../manifests/pi.json"),
        include_str!("../../manifests/picoclaw.json"),
        include_str!("../../manifests/nanobot.json"),
        include_str!("../../manifests/zeroclaw.json"),
        include_str!("../../manifests/claudedesktop.json"),
        include_str!("../../manifests/codexdesktop.json"),
        include_str!("../../manifests/geminidesktop.json"),
        include_str!("../../manifests/vscode.json"),
        include_str!("../../manifests/trae.json"),
        include_str!("../../manifests/traecn.json"),
    ];

    manifests
        .iter()
        .filter_map(|json| {
            match load_manifest(json) {
                Ok(adapter) => Some(Box::new(adapter) as Box<dyn Plugin>),
                Err(e) => {
                    eprintln!("[AIDAGuard] Failed to load manifest: {}", e);
                    None
                }
            }
        })
        .collect()
}
