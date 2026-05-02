use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tracing::error;

/// 构建系统托盘图标和菜单。
pub fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let start_item = MenuItemBuilder::with_id("start", "启动代理").build(app)?;
    let stop_item = MenuItemBuilder::with_id("stop", "停止代理").build(app)?;
    let status_item = MenuItemBuilder::with_id("status", "代理：已停止").build(app)?;
    let stats_item = MenuItemBuilder::with_id("stats", "检测: 0 | 规则: 0").build(app)?;
    let show_item = MenuItemBuilder::with_id("show", "打开主窗口").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出 Aidaguard").build(app)?;

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
                            error!("启动代理失败: {}", e);
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
