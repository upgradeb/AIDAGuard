pub mod commands;
pub mod events;
pub mod state;
pub mod tray;

pub use state::AppState;

/// 解析规则目录：绝对路径直接用，相对路径依次尝试 CWD → 可执行文件上溯 → config 目录
pub fn resolve_rules_dir(rules_dir: &str, config_dir: &std::path::Path) -> String {
    use std::path::Path;

    if Path::new(rules_dir).is_absolute() {
        return rules_dir.to_string();
    }

    // 1) 尝试当前工作目录
    let cwd_path = std::env::current_dir()
        .unwrap_or_default()
        .join(rules_dir);
    if cwd_path.exists() {
        return cwd_path.to_string_lossy().to_string();
    }

    // 2) 从可执行文件位置向上搜索（覆盖 cargo tauri dev 场景）
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

    // 3) 回退到 config 目录
    config_dir
        .join(rules_dir)
        .to_string_lossy()
        .to_string()
}
