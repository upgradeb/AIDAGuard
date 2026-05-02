/// 代理引擎
/// 负责：监听本地端口、转发请求、处理 SSE 流式响应

pub mod server;
pub mod forwarder;
pub mod stream;

pub use server::{start, start_with_state};

use serde::{Deserialize, Serialize};

/// 检测事件，用于从代理任务向 Tauri 前端广播
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvent {
    pub timestamp_ms: i64,
    pub rule_id: String,
    pub strategy: String,
    pub placeholder: String,
    pub request_path: String,
    pub response_status: u16,
}
