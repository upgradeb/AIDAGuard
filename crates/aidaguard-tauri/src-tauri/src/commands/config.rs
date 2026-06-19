use aidaguard_core::config::{Config, DetectionRegion};
use aidaguard_core::DetectionEngine;
use tauri::Manager;
use crate::state::AppState;

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn get_config(
    state: tauri::State<'_, AppState>,
) -> Result<Config, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: Config,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let _ = std::fs::create_dir_all(&config_dir);
    let path = config_dir.join("config.toml");

    // Sync rules_dir to runtime state
    let rules_dir = crate::resolve_rules_dir(&config.rules_dir, &config_dir);
    *state.rules_dir.write().await = rules_dir.clone();

    // Reload rules with updated presets if region or industries changed
    let presets = config.rule_presets();
    let rules_path = std::path::Path::new(&rules_dir);
    if rules_path.exists() {
        let mut engine = state.detector.write().await;
        let _ = engine.reload_presets(rules_path, &presets);
    }

    config.save_to(&path).map_err(|e| format!("Failed to save config: {}", e))?;

    // Update runtime state
    *state.config.write().await = config;

    Ok(())
}

/// Get all available detection regions.
#[tauri::command]
pub fn get_available_regions() -> Vec<RegionInfo> {
    DetectionRegion::available_regions()
        .into_iter()
        .map(|(code, name)| RegionInfo {
            code: code.to_string(),
            name: name.to_string(),
        })
        .collect()
}

/// Update detection region configuration and reload rules.
#[tauri::command]
pub async fn update_detection_region(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    primary_region: String,
    additional_regions: Vec<String>,
) -> Result<(), String> {
    let mut config = state.config.read().await.clone();
    config.detection_region.primary_region = primary_region;
    config.detection_region.additional_regions = additional_regions;

    // Reload rules with updated presets
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let rules_dir = crate::resolve_rules_dir(&config.rules_dir, &config_dir);
    let presets = config.rule_presets();
    let rules_path = std::path::Path::new(&rules_dir);
    if rules_path.exists() {
        let mut engine = state.detector.write().await;
        engine.reload_presets(rules_path, &presets)
            .map_err(|e| format!("Failed to reload rules: {}", e))?;
    }

    // Save and update runtime state
    let path = config_dir.join("config.toml");
    config.save_to(&path).map_err(|e| format!("Failed to save config: {}", e))?;
    *state.config.write().await = config;

    Ok(())
}

/// Region information for the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionInfo {
    pub code: String,
    pub name: String,
}
