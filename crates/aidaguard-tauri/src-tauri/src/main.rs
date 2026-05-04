use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};
use tracing_subscriber::EnvFilter;

use aidaguard_core::config::Config;
use aidaguard_core::detector::Detector;
use aidaguard_core::storage::Storage;

use aidaguard_tauri::state::AppState;
use aidaguard_tauri::{commands, resolve_rules_dir, resolve_storage_path, tray};

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // 解析配置目录
            let config_dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let _ = std::fs::create_dir_all(&config_dir);
            let config_path = config_dir.join("config.toml");

            // 加载配置
            let config = Config::load_from(&config_path).unwrap_or_default();

            // 从配置中读取规则目录
            let rules_dir = resolve_rules_dir(&config.rules_dir, &config_dir);

            // 初始化存储（无需代理运行即可查看审计记录）
            let storage: Option<Arc<Storage>> = if config.storage.enabled {
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
                        tracing::info!("存储已打开: {}", db_path);
                        Some(Arc::new(s))
                    }
                    Err(e) => {
                        tracing::warn!("存储打开失败: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let port = config.port;

            // 初始化共享状态
            let state = AppState {
                config: Arc::new(RwLock::new(config)),
                detector: Arc::new(RwLock::new(Detector::new())),
                storage: Arc::new(Mutex::new(storage)),
                proxy_handle: Arc::new(Mutex::new(None)),
                proxy_shutdown: Arc::new(Mutex::new(None)),
                proxy_start_time: Arc::new(Mutex::new(None)),
                proxy_port: Arc::new(Mutex::new(port)),
                rules_dir: Arc::new(RwLock::new(rules_dir)),
                rules_watcher: Arc::new(Mutex::new(None)),
            };

            app.manage(state);

            // 构建系统托盘
            tray::build_tray(app)?;

            // 关闭窗口时最小化到托盘，而非退出
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::proxy::start_proxy,
            commands::proxy::stop_proxy,
            commands::proxy::get_proxy_status,
            commands::audit::list_audit,
            commands::audit::list_audit_groups,
            commands::audit::get_audit_detail,
            commands::audit::delete_audit,
            commands::audit::export_audit,
            commands::audit::get_audit_stats,
            commands::audit::get_recent_events,
            commands::rules::get_rules,
            commands::rules::save_rule,
            commands::rules::delete_rule,
            commands::rules::toggle_rule,
            commands::rules::test_rule,
            commands::rules::reload_rules,
            commands::rules::get_rule_files,
            commands::rules::create_category,
            commands::rules::delete_category,
            commands::rules::rename_category,
            commands::rules::generate_rule,
            commands::config::get_config,
            commands::config::save_config,
            commands::upstream::get_upstreams,
            commands::upstream::add_upstream,
            commands::upstream::update_upstream,
            commands::upstream::delete_upstream,
            commands::upstream::set_default_upstream,
            commands::upstream::test_upstream_connectivity,
            commands::tools::detect_tools,
            commands::tools::apply_tool_config,
            commands::tools::restore_tool_config,
            commands::tools::restore_all_tools,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Aidaguard 失败");
}
