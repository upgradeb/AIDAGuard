use std::collections::HashMap;
use std::time::Instant;

use aidaguard_core::config::NotificationConfig;
use aidaguard_core::DetectionEvent;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use tracing::warn;

/// 将代理内部的检测事件转发给 Tauri 前端，并按配置发送桌面通知。
pub async fn relay_events(
    app_handle: tauri::AppHandle,
    mut rx: tokio::sync::broadcast::Receiver<DetectionEvent>,
    notify_cfg: NotificationConfig,
) {
    // 频率限制：记录每个 rule_id 最近一次通知时间
    let mut last_notify: HashMap<String, Instant> = HashMap::new();

    loop {
        match rx.recv().await {
            Ok(event) => {
                // 始终转发给前端（事件流展示）
                let _ = app_handle.emit("detection-event", &event);

                // 桌面通知
                if notify_cfg.enabled {
                    let now = Instant::now();
                    let cooldown = std::time::Duration::from_secs(notify_cfg.rate_limit_secs);
                    let can_notify = last_notify
                        .get(&event.rule_id)
                        .map(|last| now.duration_since(*last) >= cooldown)
                        .unwrap_or(true);

                    if can_notify {
                        last_notify.insert(event.rule_id.clone(), now);

                        let title = format!("Aidaguard — {}", &event.rule_id);
                        let body = format!(
                            "策略: {} | 路径: {} | 状态: {}",
                            event.strategy, event.request_path, event.response_status
                        );

                        let _ = app_handle
                            .notification()
                            .builder()
                            .title(&title)
                            .body(&body)
                            .show();
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!("事件转发滞后 {} 条消息", n);
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}
