use aidaguard_core::config::UpstreamConfig;
use tauri::{AppHandle, Manager};
use crate::state::AppState;

#[tauri::command]
pub async fn get_upstreams(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<UpstreamConfig>, String> {
    let config = state.config.read().await;
    Ok(config.upstreams.clone())
}

#[tauri::command]
pub async fn add_upstream(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    upstream: UpstreamConfig,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let path = config_dir.join("config.toml");

    let mut config = state.config.write().await;
    config.upstreams.push(upstream);
    config
        .save_to(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_upstream(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
    upstream: UpstreamConfig,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let path = config_dir.join("config.toml");

    let mut config = state.config.write().await;
    if let Some(pos) = config.upstreams.iter().position(|u| u.name == name) {
        config.upstreams[pos] = upstream;
    } else {
        config.upstreams.push(upstream);
    }
    config
        .save_to(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_upstream(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let path = config_dir.join("config.toml");

    let mut config = state.config.write().await;
    config.upstreams.retain(|u| u.name != name);
    config
        .save_to(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn set_default_upstream(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;
    let path = config_dir.join("config.toml");

    let mut config = state.config.write().await;
    for u in &mut config.upstreams {
        u.default = u.name == name;
    }
    config
        .save_to(&path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn test_upstream_connectivity(
    url: String,
    api_key: String,
    timeout_secs: u64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let auth_value = if api_key.starts_with("Bearer ") {
        api_key.clone()
    } else {
        format!("Bearer {}", api_key)
    };

    let body = serde_json::json!({
        "model": "test",
        "messages": [{"role": "user", "content": "hi"}],
        "max_tokens": 1
    })
    .to_string();

    let start = std::time::Instant::now();
    match client
        .post(&url)
        .header("authorization", &auth_value)
        .header("content-type", "application/json")
        .body(body)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
    {
        Ok(resp) => {
            let elapsed_ms = start.elapsed().as_millis();
            let status = resp.status().as_u16();
            Ok(format!(
                "✓ Connected\nLatency: {}ms\nHTTP Status: {}",
                elapsed_ms, status
            ))
        }
        Err(e) => {
            let elapsed_ms = start.elapsed().as_millis();
            Err(format!("✗ Connection failed\nLatency: {}ms\nError: {}", elapsed_ms, e))
        }
    }
}
