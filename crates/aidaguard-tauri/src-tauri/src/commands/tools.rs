use tauri::Manager;
use crate::state::AppState;
use crate::tools::{self, ToolInfo};

/// 检测所有已安装的 AI 工具，返回状态列表
#[tauri::command]
pub async fn detect_tools(
    app: tauri::AppHandle,
) -> Result<Vec<ToolInfo>, String> {
    let adapters = tools::all_adapters();
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    drop(state);
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    let mut results = Vec::new();
    for adapter in &adapters {
        let installed = adapter.detect();
        let configured = adapter.is_configured();
        results.push(ToolInfo {
            tool_id: adapter.id().to_string(),
            tool_name: adapter.name().to_string(),
            installed,
            configured,
            config_path: adapter.config_path().to_string(),
            current_endpoint: adapter.current_endpoint(),
            current_model: adapter.current_model(),
            preview_endpoint: if installed && !configured {
                Some(proxy_url.clone())
            } else {
                None
            },
        });
    }
    Ok(results)
}

/// 对指定工具执行配置（含备份）
#[tauri::command]
pub async fn apply_tool_config(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<String, String> {
    let adapters = tools::all_adapters();
    let adapter = adapters.iter().find(|a| a.id() == tool_id)
        .ok_or_else(|| format!("未知工具: {}", tool_id))?;

    if !adapter.detect() {
        return Err(format!("{} 未安装", adapter.name()));
    }

    // 获取代理地址
    let state = app.state::<AppState>();
    let proxy_port = *state.proxy_port.lock().await;
    drop(state);
    let proxy_url = format!("http://127.0.0.1:{}", proxy_port);

    // 获取备份目录
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    // 先备份
    adapter.backup(&backup_dir)?;

    // 再写入新配置
    adapter.configure(&proxy_url)?;

    Ok(format!("{} 已配置为使用 Aidaguard 代理 ({})", adapter.name(), proxy_url))
}

/// 从备份恢复指定工具的原始配置
#[tauri::command]
pub async fn restore_tool_config(
    app: tauri::AppHandle,
    tool_id: String,
) -> Result<String, String> {
    let adapters = tools::all_adapters();
    let adapter = adapters.iter().find(|a| a.id() == tool_id)
        .ok_or_else(|| format!("未知工具: {}", tool_id))?;

    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;
    let backup_dir = tools::backup::backup_dir_for(&data_dir, &tool_id);

    adapter.restore(&backup_dir)?;

    Ok(format!("{} 配置已恢复", adapter.name()))
}

/// 恢复所有工具的原始配置
#[tauri::command]
pub async fn restore_all_tools(
    app: tauri::AppHandle,
) -> Result<String, String> {
    let adapters = tools::all_adapters();
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取数据目录: {}", e))?;

    let mut restored = Vec::new();
    let mut errors = Vec::new();

    for adapter in &adapters {
        let backup_dir = tools::backup::backup_dir_for(&data_dir, adapter.id());
        match adapter.restore(&backup_dir) {
            Ok(()) => restored.push(adapter.name().to_string()),
            Err(e) => {
                // 跳过"未安装"或"无备份"的错误
                if !e.contains("备份文件不存在") && !e.contains("未安装") {
                    errors.push(format!("{}: {}", adapter.name(), e));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(format!("已恢复 {} 个工具的配置", restored.len()))
    } else {
        Err(format!("部分恢复失败: {}", errors.join("; ")))
    }
}
