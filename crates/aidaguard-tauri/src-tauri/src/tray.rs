use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tracing::error;

/// Build system tray icon and menu.
pub fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let start_item = MenuItemBuilder::with_id("start", "Start Proxy").build(app)?;
    let stop_item = MenuItemBuilder::with_id("stop", "Stop Proxy").build(app)?;
    let status_item = MenuItemBuilder::with_id("status", "Proxy: Stopped").build(app)?;
    let stats_item = MenuItemBuilder::with_id("stats", "Detections: 0 | Rules: 0").build(app)?;
    let show_item = MenuItemBuilder::with_id("show", "Open Main Window").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit Aidaguard").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&status_item)
        .item(&start_item)
        .item(&stop_item)
        .separator()
        .item(&stats_item)
        .separator()
        .item(&show_item)
        .item(&quit_item)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Aidaguard")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "start" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let s = handle.state::<crate::state::AppState>();
                        let result = crate::commands::proxy::start_proxy(handle.clone(), s).await;
                        if let Err(e) = result {
                            error!("Failed to start proxy: {}", e);
                        } else {
                            let _ = handle.emit(
                                "proxy-status-changed",
                                serde_json::json!({"status": "running"}),
                            );
                        }
                    });
                }
                "stop" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let s = handle.state::<crate::state::AppState>();
                        let _ = crate::commands::proxy::stop_proxy(s).await;
                        let _ = handle.emit(
                            "proxy-status-changed",
                            serde_json::json!({"status": "stopped"}),
                        );
                    });
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    let handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let s = handle.state::<crate::state::AppState>();
                        let _ = crate::commands::proxy::stop_proxy(s).await;
                        handle.exit(0);
                    });
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
