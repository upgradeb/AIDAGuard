pub mod commands;
pub mod events;
pub mod state;
pub mod tray;

pub use state::AppState;

/// Resolve storage database path: absolute paths used directly, relative paths try CWD → config directory
pub fn resolve_storage_path(db_path: &str, config_dir: &std::path::Path) -> String {
    use std::path::Path;

    if Path::new(db_path).is_absolute() {
        return db_path.to_string();
    }

    let cwd_path = std::env::current_dir()
        .unwrap_or_default()
        .join(db_path);
    if cwd_path.parent().map_or(false, |p| p.exists()) {
        return cwd_path.to_string_lossy().to_string();
    }

    config_dir
        .join(db_path)
        .to_string_lossy()
        .to_string()
}

/// Resolve rules directory: absolute paths used directly, relative paths resolve to config directory first,
/// then CWD → executable ancestor for dev mode compatibility.
pub fn resolve_rules_dir(rules_dir: &str, config_dir: &std::path::Path) -> String {
    use std::path::Path;

    if Path::new(rules_dir).is_absolute() {
        return rules_dir.to_string();
    }

    // 1) Try config directory (primary location — integrates rules with app data)
    let config_path = config_dir.join(rules_dir);
    if config_path.exists() {
        return config_path.to_string_lossy().to_string();
    }

    // 2) Try current working directory (cargo tauri dev scenario)
    let cwd_path = std::env::current_dir()
        .unwrap_or_default()
        .join(rules_dir);
    if cwd_path.exists() {
        return cwd_path.to_string_lossy().to_string();
    }

    // 3) Search upward from executable location
    if let Ok(exe) = std::env::current_exe() {
        let mut exe_dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        loop {
            let candidate = exe_dir.join(rules_dir);
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
            if !exe_dir.pop() {
                break;
            }
        }
    }

    // 4) Return config directory path (will be created on first use)
    config_path.to_string_lossy().to_string()
}
