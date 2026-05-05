use tauri::Manager;
use crate::state::AppState;
use crate::tools::{self, ToolInfo};

/// Detect all installed AI tools and return their status
#[tauri::command]
pub async fn detect_tools(
    app: tauri::AppHandle,
) -> Result<Vec<ToolInfo>, String> {
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    let registry = state.plugin_registry.read().await;
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    let mut results = Vec::new();
    for plugin in registry.iter() {
        let manifest = plugin.manifest();
        let installed = plugin.detect();
        let configured = plugin.is_configured();
        let enabled = registry.is_enabled(plugin.id());
        results.push(ToolInfo {
            tool_id: plugin.id().to_string(),
            tool_name: plugin.name().to_string(),
            installed,
            configured,
            config_path: plugin.config_path().to_string(),
            current_endpoint: plugin.current_endpoint(),
            current_model: plugin.current_model(),
            preview_endpoint: if installed && !configured {
                Some(proxy_url.clone())
            } else {
                None
            },
            version: manifest.version,
            description: manifest.description,
            author: manifest.author,
            categories: manifest.categories,
            enabled,
        });
    }
    Ok(results)
}

/// Apply configuration to a specific tool (with backup)
#[tauri::command]
pub async fn apply_tool_config(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    let registry = state.plugin_registry.read().await;
    let plugin = registry.get(&tool_id)
        .ok_or_else(|| format!("Unknown tool: {}", tool_id))?;

    if !plugin.detect() {
        return Err(format!("{} is not installed", plugin.name()));
    }

    plugin.backup(&backup_dir)?;
    plugin.configure(&proxy_url)?;

    Ok(format!("{} configured to use Aidaguard proxy ({})", plugin.name(), proxy_url))
}

/// Restore original config for a specific tool from backup
#[tauri::command]
pub async fn restore_tool_config(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<String, String> {
    let state = app.state::<AppState>();

    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    if !backup_dir.exists() || std::fs::read_dir(&backup_dir).map(|mut d| d.next().is_none()).unwrap_or(true) {
        return Err(format!("{} has no backup. Please run \"Configure\" first to create a backup.", tool_id));
    }

    let registry = state.plugin_registry.read().await;
    let plugin = registry.get(&tool_id)
        .ok_or_else(|| format!("Unknown tool: {}", tool_id))?;

    if !plugin.detect() {
        return Err(format!("{} is not installed", plugin.name()));
    }

    plugin.restore(&backup_dir)?;

    Ok(format!("{} configuration restored", plugin.name()))
}

/// Restore original config for all tools
#[tauri::command]
pub async fn restore_all_tools(
    app: tauri::AppHandle,
) -> Result<String, String> {
    let state = app.state::<AppState>();
    let registry = state.plugin_registry.read().await;
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;

    let mut restored = Vec::new();
    let mut errors = Vec::new();

    for plugin in registry.iter() {
        let backup_dir = tools::backup::backup_dir_for(&data_dir, plugin.id());
        match plugin.restore(&backup_dir) {
            Ok(()) => restored.push(plugin.name().to_string()),
            Err(e) => {
                if !e.contains("Backup file does not exist") && !e.contains("is not installed") {
                    errors.push(format!("{}: {}", plugin.name(), e));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(format!("Restored configuration for {} tool(s)", restored.len()))
    } else {
        Err(format!("Partial restore failure: {}", errors.join("; ")))
    }
}

/// Enable a plugin by id
#[tauri::command]
pub async fn enable_plugin(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut registry = state.plugin_registry.write().await;
    registry.enable(&tool_id)
}

/// Disable a plugin by id
#[tauri::command]
pub async fn disable_plugin(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut registry = state.plugin_registry.write().await;
    registry.disable(&tool_id)
}
