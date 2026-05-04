use std::collections::HashMap;
use std::time::Instant;

use aidaguard_core::config::NotificationConfig;
use aidaguard_core::DetectionEvent;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use tracing::warn;

/// Forward proxy detection events to the Tauri frontend and send desktop notifications per configuration.
pub async fn relay_events(
    app_handle: tauri::AppHandle,
    mut rx: tokio::sync::broadcast::Receiver<DetectionEvent>,
    notify_cfg: NotificationConfig,
) {
    // Rate limit: track last notification time per rule_id
    let mut last_notify: HashMap<String, Instant> = HashMap::new();

    loop {
        match rx.recv().await {
            Ok(event) => {
                // Always forward to frontend (event stream display)
                let _ = app_handle.emit("detection-event", &event);

                // Desktop notification
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
                            "Strategy: {} | Path: {} | Status: {}",
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
                warn!("Event relay lagged by {} messages", n);
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}
