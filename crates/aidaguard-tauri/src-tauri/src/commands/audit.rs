use serde::Serialize;

use aidaguard_core::storage::{AuditGroup, AuditStats, DetectionRecord};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditListResponse {
    pub records: Vec<DetectionRecord>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditGroupResponse {
    pub groups: Vec<AuditGroup>,
    pub total: usize,
}

#[tauri::command]
pub async fn list_audit(
    state: tauri::State<'_, AppState>,
    limit: usize,
    offset: usize,
    rule_id_filter: Option<String>,
    path_filter: Option<String>,
    date_from_ms: Option<i64>,
    date_to_ms: Option<i64>,
    strategy_filter: Option<String>,
) -> Result<AuditListResponse, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    let records = storage
        .list_filtered(
            limit,
            offset,
            rule_id_filter.as_deref(),
            path_filter.as_deref(),
            date_from_ms,
            date_to_ms,
            strategy_filter.as_deref(),
        )
        .map_err(|e| format!("查询审计记录失败: {}", e))?;

    let total = storage
        .count_filtered(
            rule_id_filter.as_deref(),
            path_filter.as_deref(),
            date_from_ms,
            date_to_ms,
            strategy_filter.as_deref(),
        )
        .map_err(|e| format!("查询总数失败: {}", e))?;

    Ok(AuditListResponse { records, total })
}

#[tauri::command]
pub async fn list_audit_groups(
    state: tauri::State<'_, AppState>,
    limit: usize,
    offset: usize,
    rule_id_filter: Option<String>,
    path_filter: Option<String>,
    date_from_ms: Option<i64>,
    date_to_ms: Option<i64>,
) -> Result<AuditGroupResponse, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    let groups = storage
        .list_grouped(
            limit,
            offset,
            rule_id_filter.as_deref(),
            path_filter.as_deref(),
            date_from_ms,
            date_to_ms,
        )
        .map_err(|e| format!("查询审计分组失败: {}", e))?;

    let total = storage
        .count_grouped(
            rule_id_filter.as_deref(),
            path_filter.as_deref(),
            date_from_ms,
            date_to_ms,
        )
        .map_err(|e| format!("查询分组总数失败: {}", e))?;

    Ok(AuditGroupResponse { groups, total })
}

#[tauri::command]
pub async fn get_audit_detail(
    state: tauri::State<'_, AppState>,
    record_id: String,
) -> Result<Option<DetectionRecord>, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    storage
        .get_by_id(&record_id)
        .map_err(|e| format!("查询详情失败: {}", e))
}

#[tauri::command]
pub async fn delete_audit(
    state: tauri::State<'_, AppState>,
    record_id: String,
) -> Result<bool, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    storage
        .delete(&record_id)
        .map_err(|e| format!("删除失败: {}", e))
}

#[tauri::command]
pub async fn export_audit(
    state: tauri::State<'_, AppState>,
    format: String,
    rule_id_filter: Option<String>,
    date_from_ms: Option<i64>,
    date_to_ms: Option<i64>,
) -> Result<String, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    // 一次最多导出 10000 条
    let records = storage
        .list_filtered(10000, 0, rule_id_filter.as_deref(), None, date_from_ms, date_to_ms, None)
        .map_err(|e| format!("导出查询失败: {}", e))?;

    if records.is_empty() {
        return Err("没有可导出的记录".into());
    }

    // 确定导出路径
    let dir = dirs_next().unwrap_or_else(|| std::path::PathBuf::from("."));
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ext = if format == "csv" { "csv" } else { "json" };
    let file_path = dir.join(format!("aidaguard_export_{}.{}", timestamp, ext));

    match format.as_str() {
        "csv" => {
            let mut wtr = csv::Writer::from_path(&file_path)
                .map_err(|e| format!("创建 CSV 文件失败: {}", e))?;
            wtr.write_record(&[
                "id", "timestamp_ms", "rule_id", "strategy", "placeholder",
                "request_path", "response_status",
            ])
            .map_err(|e| format!("CSV 写入失败: {}", e))?;
            for r in &records {
                wtr.write_record(&[
                    &r.id,
                    &r.timestamp_ms.to_string(),
                    &r.rule_id,
                    &r.strategy,
                    &r.placeholder,
                    &r.request_path,
                    &r.response_status.to_string(),
                ])
                .map_err(|e| format!("CSV 写入失败: {}", e))?;
            }
            wtr.flush().map_err(|e| format!("CSV flush 失败: {}", e))?;
        }
        "json" => {
            let export: Vec<serde_json::Value> = records
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "timestamp_ms": r.timestamp_ms,
                        "rule_id": r.rule_id,
                        "strategy": r.strategy,
                        "placeholder": r.placeholder,
                        "request_path": r.request_path,
                        "response_status": r.response_status,
                    })
                })
                .collect();
            let json_str =
                serde_json::to_string_pretty(&export).map_err(|e| format!("JSON 序列化失败: {}", e))?;
            std::fs::write(&file_path, json_str)
                .map_err(|e| format!("写入文件失败: {}", e))?;
        }
        _ => return Err(format!("不支持的导出格式: {}", format)),
    }

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_audit_stats(
    state: tauri::State<'_, AppState>,
) -> Result<AuditStats, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    storage.stats().map_err(|e| format!("统计查询失败: {}", e))
}

#[tauri::command]
pub async fn get_recent_events(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DetectionRecord>, String> {
    let storage = state.storage.lock().await;
    let storage = storage
        .as_ref()
        .ok_or_else(|| "审计存储未启用".to_string())?;

    storage.list_recent(5).map_err(|e| format!("查询最近事件失败: {}", e))
}

fn dirs_next() -> Option<std::path::PathBuf> {
    // Try common download directories
    if let Ok(dir) = std::env::var("HOME") {
        let downloads = std::path::PathBuf::from(dir).join("Downloads");
        if downloads.exists() {
            return Some(downloads);
        }
    }
    if let Some(dir) = dirs_next_impl() {
        return Some(dir);
    }
    None
}

fn dirs_next_impl() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let desktop = std::path::PathBuf::from(home).join("Desktop");
            if desktop.exists() {
                return Some(desktop);
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let desktop = std::path::PathBuf::from(profile).join("Desktop");
            if desktop.exists() {
                return Some(desktop);
            }
        }
    }
    None
}
