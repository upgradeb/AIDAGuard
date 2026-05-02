use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

use aidaguard_core::detector::watch_rules;
use aidaguard_core::proxy::start_with_state;
use aidaguard_core::storage::Storage;
use aidaguard_core::DetectionEvent;

use crate::events;
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
    // 检查是否已在运行
    {
        let handle = state.proxy_handle.lock().await;
        if handle.is_some() {
            return Err("代理已在运行中".into());
        }
    }

    let mut config = state.config.read().await.clone();

    // 从默认上游解析 target_url 和 api_key
    if config.target_url.is_empty() {
        if let Some(up) = config.upstreams.iter().find(|u| u.default) {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
        } else if let Some(up) = config.upstreams.first() {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
        }
    }

    // 加载规则
    let mut detector = state.detector.write().await;
    let rules_path = std::path::Path::new(&config.rules_dir);
    if rules_path.exists() {
        detector
            .load_from_dir(rules_path)
            .map_err(|e| format!("规则加载失败: {}", e))?;
    } else {
        warn!("规则目录不存在: {}", config.rules_dir);
    }
    let _rules_count = detector.rule_count();
    drop(detector);

    // 打开存储
    let storage = if config.storage.enabled {
        if let Some(parent) = std::path::Path::new(&config.storage.db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let enc_key = config
            .storage
            .encryption_key
            .as_deref()
            .unwrap_or("aidaguard-internal-key");
        match Storage::open(std::path::Path::new(&config.storage.db_path), enc_key) {
            Ok(s) => {
                info!("Storage enabled: {}", config.storage.db_path);
                Some(Arc::new(s))
            }
            Err(e) => {
                warn!("Storage 打开失败: {}", e);
                None
            }
        }
    } else {
        None
    };

    let storage_arc = storage.clone();
    *state.storage.lock().await = storage_arc;

    // 创建事件广播通道
    let (event_tx, event_rx) = tokio::sync::broadcast::channel::<DetectionEvent>(256);

    // 启动事件转发任务
    let app_handle = app.clone();
    tokio::spawn(async move {
        events::relay_events(app_handle, event_rx).await;
    });

    // 创建关闭信号
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

    // 启动代理任务
    let handle = tokio::spawn(async move {
        if let Err(e) = start_with_state(
            config,
            detector_clone,
            storage_clone,
            Some(event_tx),
            shutdown_signal,
        )
        .await
        {
            error!("代理运行异常: {}", e);
        }
    });

    *state.proxy_handle.lock().await = Some(handle);

    // 启动规则热加载
    let rules_dir = state.rules_dir.read().await.clone();
    let rules_path = std::path::PathBuf::from(&rules_dir);
    match watch_rules(state.detector.clone(), rules_path) {
        Ok(watcher) => {
            *state.rules_watcher.lock().await = Some(watcher);
        }
        Err(e) => {
            warn!("规则热加载启动失败: {}", e);
        }
    }

    info!("代理已启动于 127.0.0.1:{}", port);
    Ok(format!("代理已启动于 127.0.0.1:{}", port))
}

#[tauri::command]
pub async fn stop_proxy(state: tauri::State<'_, AppState>) -> Result<String, String> {
    // 发送关闭信号
    if let Some(tx) = state.proxy_shutdown.lock().await.take() {
        let _ = tx.send(());
    } else {
        return Err("代理未在运行".into());
    }

    // 等待任务完成
    if let Some(handle) = state.proxy_handle.lock().await.take() {
        match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
            Ok(Ok(())) => info!("代理任务已退出"),
            Ok(Err(e)) => error!("代理任务错误: {}", e),
            Err(_) => {
                warn!("代理关闭超时，强制退出");
            }
        }
    }

    // 清理规则热加载
    *state.rules_watcher.lock().await = None;

    // 清理状态
    *state.proxy_start_time.lock().await = None;

    info!("代理已停止");
    Ok("代理已停止".to_string())
}

#[tauri::command]
pub async fn get_proxy_status(
    state: tauri::State<'_, AppState>,
) -> Result<ProxyStatus, String> {
    let mut handle = state.proxy_handle.lock().await;
    // 检查 JoinHandle 是否已完成（任务自行退出）
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
