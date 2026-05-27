use std::fs;
use std::path::{Path, PathBuf};

/// Get tool backup directory: ~/.aidaguard/backups/{tool_id}/
pub fn backup_dir_for(data_dir: &Path, tool_id: &str) -> PathBuf {
    data_dir.join("backups").join(tool_id)
}

/// Backup tool config file to backup directory.
/// If a backup already exists, delete the old one and create a new one (keep only the latest).
pub fn backup_config(config_path: &Path, backup_dir: &Path) -> Result<(), String> {
    if !config_path.exists() {
        return Err(format!("Config file does not exist: {}", config_path.display()));
    }

    let _ = fs::create_dir_all(backup_dir);

    let file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");

    let dest = backup_dir.join(file_name);
    fs::copy(config_path, &dest)
        .map_err(|e| format!("Backup failed: {}", e))?;

    Ok(())
}

/// Restore config from backup directory to original path.
/// If no backup exists, the config was created by configure() — delete it to restore.
pub fn restore_config(config_path: &Path, backup_dir: &Path) -> Result<(), String> {
    let file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json");

    let backup_file = backup_dir.join(file_name);
    if !backup_file.exists() {
        // No backup means the config didn't exist before configure.
        // Delete the current config file to return to pre-configure state.
        if config_path.exists() {
            fs::remove_file(config_path)
                .map_err(|e| format!("Failed to remove config file (no backup to restore): {}", e))?;
        }
        return Ok(());
    }

    // Ensure target directory exists
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::copy(&backup_file, config_path)
        .map_err(|e| format!("Restore failed: {}", e))?;

    Ok(())
}
