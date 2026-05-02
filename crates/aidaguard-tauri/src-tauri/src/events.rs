use aidaguard_core::DetectionEvent;
use tauri::Emitter;
use tracing::warn;

/// 将代理内部的检测事件转发给 Tauri 前端。
pub async fn relay_events(
    app_handle: tauri::AppHandle,
    mut rx: tokio::sync::broadcast::Receiver<DetectionEvent>,
) {
    loop {
        match rx.recv().await {
            Ok(event) => {
                let _ = app_handle.emit("detection-event", &event);
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!("事件转发滞后 {} 条消息", n);
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}
