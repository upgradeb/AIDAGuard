use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tauri::Manager;
use tracing::{error, info, warn};

use aidaguard_core::detector::watch_rules;
use aidaguard_proxy::start_with_state;
use aidaguard_storage::Storage;
use aidaguard_core::DetectionEvent;

use crate::events;
use crate::resolve_storage_path;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatus {
    pub status: String,
    pub port: u16,
    pub uptime_secs: u64,
    pub rules_count: usize,
    pub storage_enabled: bool,
    pub error_message: Option<String>,
}

#[tauri::command]
pub async fn start_proxy(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // Check if already running
    {
        let handle = state.proxy_handle.lock().await;
        if handle.is_some() {
            return Err("Proxy is already running".into());
        }
    }

    let mut config = state.config.read().await.clone();

    // Resolve target_url, api_key, and upstream_name from default upstream
    let upstream_name = if config.target_url.is_empty() {
        if let Some(up) = config.upstreams.iter().find(|u| u.default) {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            up.name.clone()
        } else if let Some(up) = config.upstreams.first() {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            up.name.clone()
        } else {
            String::new()
        }
    } else {
        config.upstreams.iter()
            .find(|u| u.url == config.target_url)
            .map(|u| u.name.clone())
            .unwrap_or_default()
    };

    // Pre-start validation
    if config.target_url.is_empty() {
        return Err("Upstream URL not set. Please add an upstream in \"LLM Upstreams\" and set it as default.".into());
    }
    if config.api_key.is_empty() {
        return Err("API Key not set. Please configure an API Key for the default upstream in \"LLM Upstreams\".".into());
    }

    // Load rules (using resolved rules_dir)
    let resolved_rules_dir = state.rules_dir.read().await.clone();
    let mut detector = state.detector.write().await;
    let rules_path = std::path::Path::new(&resolved_rules_dir);
    if rules_path.exists() {
        detector
            .load_from_dir(rules_path)
            .map_err(|e| format!("Failed to load rules: {}", e))?;
    } else {
        warn!("Rules directory does not exist: {}", resolved_rules_dir);
    }
    let _rules_count = detector.rule_count();
    drop(detector);

    // Open storage (if not already open)
    let storage = if config.storage.enabled {
        let existing = state.storage.lock().await.clone();
        if existing.is_some() {
            existing
        } else {
            let config_dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let db_path = resolve_storage_path(&config.storage.db_path, &config_dir);
            if let Some(parent) = std::path::Path::new(&db_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let enc_key = config
                .storage
                .encryption_key
                .as_deref()
                .unwrap_or("aidaguard-internal-key");
            match Storage::open(std::path::Path::new(&db_path), enc_key) {
                Ok(s) => {
                    info!("Storage enabled: {}", db_path);
                    Some(Arc::new(s))
                }
                Err(e) => {
                    warn!("Failed to open storage: {}", e);
                    None
                }
            }
        }
    } else {
        None
    };

    let storage_arc = storage.clone();
    *state.storage.lock().await = storage_arc;

    // Create event broadcast channel
    let (event_tx, event_rx) = tokio::sync::broadcast::channel::<DetectionEvent>(256);

    // Start event relay task (with desktop notifications)
    let app_handle = app.clone();
    let notify_cfg = config.notification.clone();
    tokio::spawn(async move {
        events::relay_events(app_handle, event_rx, notify_cfg).await;
    });

    // Create shutdown signal
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_signal = async move {
        let _ = shutdown_rx.await;
    };

    let port = config.port;
    let detector_clone = state.detector.clone();
    let storage_clone = state.storage.lock().await.clone();

    *state.proxy_port.lock().await = port;
    *state.proxy_shutdown.lock().await = Some(shutdown_tx);
    *state.proxy_start_time.lock().await = Some(Instant::now());

    // Start proxy task
    let handle = tokio::spawn(async move {
        if let Err(e) = start_with_state(
            config,
            detector_clone,
            storage_clone,
            Some(event_tx),
            shutdown_signal,
            upstream_name,
        )
        .await
        {
            error!("Proxy runtime error: {}", e);
        }
    });

    *state.proxy_handle.lock().await = Some(handle);

    // Start rule hot-reload
    let rules_dir = state.rules_dir.read().await.clone();
    let rules_path = std::path::PathBuf::from(&rules_dir);
    match watch_rules(state.detector.clone(), rules_path) {
        Ok(watcher) => {
            *state.rules_watcher.lock().await = Some(watcher);
        }
        Err(e) => {
            warn!("Failed to start rule hot-reload: {}", e);
        }
    }

    info!("Proxy started on 127.0.0.1:{}", port);
    Ok(format!("Proxy started on 127.0.0.1:{}", port))
}

#[tauri::command]
pub async fn stop_proxy(state: tauri::State<'_, AppState>) -> Result<String, String> {
    // Send shutdown signal
    if let Some(tx) = state.proxy_shutdown.lock().await.take() {
        let _ = tx.send(());
    } else {
        return Err("Proxy is not running".into());
    }

    // Wait for task completion
    if let Some(handle) = state.proxy_handle.lock().await.take() {
        match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
            Ok(Ok(())) => info!("Proxy task exited"),
            Ok(Err(e)) => error!("Proxy task error: {}", e),
            Err(_) => {
                warn!("Proxy shutdown timed out, forcing exit");
            }
        }
    }

    // Clean up rule hot-reload
    *state.rules_watcher.lock().await = None;

    // Clean up state
    *state.proxy_start_time.lock().await = None;

    info!("Proxy stopped");
    Ok("Proxy stopped".to_string())
}

#[tauri::command]
pub async fn get_proxy_status(
    state: tauri::State<'_, AppState>,
) -> Result<ProxyStatus, String> {
    let mut handle = state.proxy_handle.lock().await;
    // Check if JoinHandle has finished (task exited on its own)
    if handle.as_ref().is_some_and(|h| h.is_finished()) {
        *handle = None;
    }
    let is_running = handle.is_some();
    drop(handle);
    let port = *state.proxy_port.lock().await;
    let uptime = state
        .proxy_start_time
        .lock()
        .await
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);
    let rules_count = state.detector.read().await.rule_count();
    let storage_enabled = state.storage.lock().await.is_some();

    Ok(ProxyStatus {
        status: if is_running {
            "running".into()
        } else {
            "stopped".into()
        },
        port,
        uptime_secs: uptime,
        rules_count,
        storage_enabled,
        error_message: None,
    })
}
