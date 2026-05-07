use aidaguard_core::config::Config;
use aidaguard_core::detector::Detector;
use aidaguard_storage::Storage;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use aidaguard_plugins::PluginRegistry;

/// Tauri-managed application state shared between proxy tasks and Tauri commands.
pub struct AppState {
    /// Current configuration
    pub config: Arc<RwLock<Config>>,
    /// Rule detector
    pub detector: Arc<RwLock<Detector>>,
    /// Audit storage (None means not enabled)
    pub storage: Arc<Mutex<Option<Arc<Storage>>>>,
    /// Proxy task handle
    pub proxy_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Shutdown signal sender
    pub proxy_shutdown: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// Proxy start time
    pub proxy_start_time: Arc<Mutex<Option<Instant>>>,
    /// Proxy listen port
    pub proxy_port: Arc<Mutex<u16>>,
    /// Rules directory path
    pub rules_dir: Arc<RwLock<String>>,
    /// Rule file hot-reload watcher (must be held to stay alive)
    pub rules_watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
    /// Plugin registry for AI tool adapters
    pub plugin_registry: Arc<RwLock<PluginRegistry>>,
}
