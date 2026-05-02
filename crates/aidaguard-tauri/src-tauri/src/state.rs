use aidaguard_core::config::Config;
use aidaguard_core::detector::Detector;
use aidaguard_core::storage::Storage;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

/// Tauri 托管的应用状态，在代理任务和 Tauri 命令之间共享。
pub struct AppState {
    /// 当前配置
    pub config: Arc<RwLock<Config>>,
    /// 规则检测器
    pub detector: Arc<RwLock<Detector>>,
    /// 审计存储（None 表示未启用）
    pub storage: Arc<Mutex<Option<Arc<Storage>>>>,
    /// 代理任务句柄
    pub proxy_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// 关闭信号发送端
    pub proxy_shutdown: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// 代理启动时间
    pub proxy_start_time: Arc<Mutex<Option<Instant>>>,
    /// 代理监听端口
    pub proxy_port: Arc<Mutex<u16>>,
    /// 规则目录路径
    pub rules_dir: Arc<RwLock<String>>,
    /// 规则文件热加载 watcher（需持有以保持运行）
    pub rules_watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
}
