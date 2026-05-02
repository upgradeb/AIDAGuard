use aidaguard_core::config::Config;
use tauri::Manager;
use crate::state::AppState;

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
        .map_err(|e| format!("无法获取配置目录: {}", e))?;
    let _ = std::fs::create_dir_all(&config_dir);
    let path = config_dir.join("config.toml");

    config.save_to(&path).map_err(|e| format!("保存配置失败: {}", e))?;

    // 更新运行时状态
    *state.config.write().await = config;

    Ok(())
}
