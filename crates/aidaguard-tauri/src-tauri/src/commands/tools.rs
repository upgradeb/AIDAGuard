use tauri::Manager;
use crate::state::AppState;
use crate::tools::{self, ToolInfo};

/// Detect all installed AI tools and return their status
#[tauri::command]
pub async fn detect_tools(
    app: tauri::AppHandle,
) -> Result<Vec<ToolInfo>, String> {
    let adapters = tools::all_adapters();
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    drop(state);
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    let mut results = Vec::new();
    for adapter in &adapters {
        let installed = adapter.detect();
        let configured = adapter.is_configured();
        results.push(ToolInfo {
            tool_id: adapter.id().to_string(),
            tool_name: adapter.name().to_string(),
            installed,
            configured,
            config_path: adapter.config_path().to_string(),
            current_endpoint: adapter.current_endpoint(),
            current_model: adapter.current_model(),
            preview_endpoint: if installed && !configured {
                Some(proxy_url.clone())
            } else {
                None
            },
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
    let adapters = tools::all_adapters();
    let adapter = adapters.iter().find(|a| a.id() == tool_id)
        .ok_or_else(|| format!("Unknown tool: {}", tool_id))?;

    if !adapter.detect() {
        return Err(format!("{} is not installed", adapter.name()));
    }

    // Get proxy address
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    drop(state);
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    // Get backup directory
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    // Backup first
    adapter.backup(&backup_dir)?;

    // Then write new config
    adapter.configure(&proxy_url)?;

    Ok(format!("{} configured to use Aidaguard proxy ({})", adapter.name(), proxy_url))
}

/// Restore original config for a specific tool from backup
#[tauri::command]
pub async fn restore_tool_config(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<String, String> {
    let adapters = tools::all_adapters();
    let adapter = adapters.iter().find(|a| a.id() == tool_id)
        .ok_or_else(|| format!("Unknown tool: {}", tool_id))?;

    if !adapter.detect() {
        return Err(format!("{} is not installed", adapter.name()));
    }

    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    // Check if backup exists
    if !backup_dir.exists() || std::fs::read_dir(&backup_dir).map(|mut d| d.next().is_none()).unwrap_or(true) {
        return Err(format!("{} has no backup. Please run \"Configure\" first to create a backup.", adapter.name()));
    }

    adapter.restore(&backup_dir)?;

    Ok(format!("{} configuration restored", adapter.name()))
}

/// Restore original config for all tools
#[tauri::command]
pub async fn restore_all_tools(
    app: tauri::AppHandle,
) -> Result<String, String> {
    let adapters = tools::all_adapters();
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get data directory: {}", e))?;

    let mut restored = Vec::new();
    let mut errors = Vec::new();

    for adapter in &adapters {
        let backup_dir = tools::backup::backup_dir_for(&data_dir, adapter.id());
        match adapter.restore(&backup_dir) {
            Ok(()) => restored.push(adapter.name().to_string()),
            Err(e) => {
                // Skip "not installed" or "no backup" errors
                if !e.contains("Backup file does not exist") && !e.contains("is not installed") {
                    errors.push(format!("{}: {}", adapter.name(), e));
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
