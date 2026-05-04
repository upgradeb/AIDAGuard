use std::fs;
use std::path::{Path, PathBuf};

/// 获取工具备份目录：~/.aidaguard/backups/{tool_id}/
pub fn backup_dir_for(data_dir: &Path, tool_id: &str) -> PathBuf {
    data_dir.join("backups").join(tool_id)
}

/// 备份工具的配置文件到备份目录。
/// 如果已存在备份，先删除旧的再创建新的（仅保留最新一份备份）。
pub fn backup_config(config_path: &Path, backup_dir: &Path) -> Result<(), String> {
    if !config_path.exists() {
        return Err(format!("配置文件不存在: {}", config_path.display()));
    }

    let _ = fs::create_dir_all(backup_dir);

    let file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");

    let dest = backup_dir.join(file_name);
    fs::copy(config_path, &dest)
        .map_err(|e| format!("备份失败: {}", e))?;

    Ok(())
}

/// 从备份目录恢复配置到原始路径。
pub fn restore_config(config_path: &Path, backup_dir: &Path) -> Result<(), String> {
    let file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");

    let backup_file = backup_dir.join(file_name);
    if !backup_file.exists() {
        return Err(format!("备份文件不存在: {}", backup_file.display()));
    }

    // 确保目标目录存在
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::copy(&backup_file, config_path)
        .map_err(|e| format!("恢复失败: {}", e))?;

    Ok(())
}
