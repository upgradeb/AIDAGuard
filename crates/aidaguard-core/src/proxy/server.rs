use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
    routing::{any, get, post},
    Json, Router,
};
use reqwest::Method;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::detector::Detector;
use crate::replacer::{self, PlaceholderMap};
use crate::storage::Storage;
use super::forwarder::Forwarder;
use super::stream;
use super::DetectionEvent;

/// 代理共享状态
#[derive(Clone)]
struct ProxyState {
    forwarder: Forwarder,
    api_key: Arc<String>,
    target_url: Arc<String>,
    upstream_name: Arc<String>,
    detector: Arc<RwLock<Detector>>,
    storage: Option<Arc<Storage>>,
    start_time: Instant,
    version: &'static str,
    max_body_size: usize,
    rules_dir: Arc<String>,
    event_tx: Option<tokio::sync::broadcast::Sender<DetectionEvent>>,
}

/// 启动代理服务器（便捷包装，用于 CLI 独立运行）。
/// 内部创建 Detector / Storage 并永不主动关闭。
pub async fn start(mut config: Config) -> Result<(), anyhow::Error> {
    // 从默认上游解析 target_url、api_key 和 upstream_name（与 Tauri start_proxy 保持一致）
    let upstream_name = if config.target_url.is_empty() {
        if let Some(up) = config.upstreams.iter().find(|u| u.default) {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            up.name.clone()
        } else if let Some(up) = config.upstreams.first() {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            up.name.clone()
        } else {
            String::new()
        }
    } else {
        config.upstreams.iter()
            .find(|u| u.url == config.target_url)
            .map(|u| u.name.clone())
            .unwrap_or_default()
    };

    let mut detector = Detector::new();
    let rules_path = std::path::Path::new(&config.rules_dir);
    if rules_path.exists() {
        detector.load_from_dir(rules_path)?;
    } else {
        warn!("规则目录不存在: {}", config.rules_dir);
    }
    let detector = Arc::new(RwLock::new(detector));

    let storage = open_storage(&config);

    start_with_state(config, detector, storage, None, std::future::pending(), upstream_name).await
}

/// 启动代理服务器，接受外部管理的 Detector / Storage 及关闭信号。
///
/// * `config` — 代理配置
/// * `detector` — 外部管理的检测器（与 Tauri 命令共享）
/// * `storage` — 外部管理的审计存储（与 Tauri 命令共享）
/// * `event_tx` — 可选，检测事件广播通道
/// * `shutdown_signal` — 触发后开始优雅关闭（axum graceful shutdown）
pub async fn start_with_state<F>(
    config: Config,
    detector: Arc<RwLock<Detector>>,
    storage: Option<Arc<Storage>>,
    event_tx: Option<tokio::sync::broadcast::Sender<DetectionEvent>>,
    shutdown_signal: F,
    upstream_name: String,
) -> Result<(), anyhow::Error>
where
    F: Future<Output = ()> + Send + 'static,
{
    let api_key = if config.api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "API Key 未设置，请在 config.toml 中配置 api_key"
        ));
    } else {
        config.api_key.clone()
    };

    info!("API Key 已加载（前8位：{}...）", &api_key[..8.min(api_key.len())]);
    info!("Target URL: {}", config.target_url);
    info!("Rules dir: {}", config.rules_dir);

    let forwarder = Forwarder::new()?;

    let rules_dir = Arc::new(config.rules_dir.clone());
    let max_body_size = config.max_body_size_mb * 1024 * 1024;

    let state = ProxyState {
        forwarder,
        api_key: Arc::new(api_key),
        target_url: Arc::new(config.target_url),
        upstream_name: Arc::new(upstream_name),
        detector: detector.clone(),
        storage,
        start_time: Instant::now(),
        version: crate::VERSION,
        max_body_size,
        rules_dir: rules_dir.clone(),
        event_tx,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/reload", post(reload_rules))
        .route("/", any(proxy_handler))
        .route("/*path", any(proxy_handler))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Aidaguard proxy listening on http://{}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("Aidaguard proxy stopped");
    Ok(())
}

fn open_storage(config: &Config) -> Option<Arc<Storage>> {
    if config.storage.enabled {
        if let Some(parent) = std::path::Path::new(&config.storage.db_path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("无法创建存储目录: {}", e);
                return None;
            }
        }
        let enc_key = config
            .storage
            .encryption_key
            .as_deref()
            .unwrap_or("aidaguard-internal-key");
        match Storage::open(std::path::Path::new(&config.storage.db_path), enc_key) {
            Ok(s) => {
                info!("Storage enabled: {}", config.storage.db_path);
                Some(Arc::new(s))
            }
            Err(e) => {
                warn!("Storage 打开失败: {}", e);
                None
            }
        }
    } else {
        None
    }
}

/// GET /health — 健康检查端点
async fn health_check(
    axum::extract::State(state): axum::extract::State<ProxyState>,
) -> Json<serde_json::Value> {
    let rules_count = state.detector.read().await.rule_count();
    Json(serde_json::json!({
        "status": "ok",
        "version": state.version,
        "uptime_seconds": state.start_time.elapsed().as_secs(),
        "rules_count": rules_count,
        "storage_enabled": state.storage.is_some(),
        "max_body_size_mb": state.max_body_size / (1024 * 1024),
    }))
}

/// POST /reload — 手动重载规则
async fn reload_rules(
    axum::extract::State(state): axum::extract::State<ProxyState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let rules_path = std::path::Path::new(state.rules_dir.as_str());
    let mut detector = state.detector.write().await;
    match detector.load_from_dir(rules_path) {
        Ok(count) => {
            info!("规则手动重载完成: {} 条", count);
            Ok(Json(serde_json::json!({
                "status": "ok",
                "rules_count": count,
                "message": format!("已加载 {} 条规则", count),
            })))
        }
        Err(e) => {
            error!("规则重载失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
            ))
        }
    }
}

/// 代理转发 handler：检测 → 替换 → 转发 → 还原
async fn proxy_handler(
    axum::extract::State(state): axum::extract::State<ProxyState>,
    req: Request,
) -> Result<Response, (StatusCode, String)> {
    // 1. 从请求头提取工具名
    let tool_name = extract_client_name(req.headers());

    // 2. 拼接目标 URL
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("")
        .to_string();

    let target_url = if path_and_query == "/" || path_and_query.is_empty() {
        state.target_url.to_string()
    } else {
        format!("{}{}", state.target_url, path_and_query)
    };

    info!("--> {} {}", req.method(), target_url);

    // 3. 处理请求头
    let method: Method = req.method().clone();
    let headers = forward_headers(req.headers());

    // 4. 检查请求体大小
    let content_length = req
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length > state.max_body_size {
        warn!(
            "请求体过大: {} bytes (限制: {} MB)",
            content_length, state.max_body_size / (1024 * 1024)
        );
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("请求体不能超过 {} MB", state.max_body_size / (1024 * 1024)),
        ));
    }

    // 5. 读取请求体
    let mut body_bytes = axum::body::to_bytes(req.into_body(), state.max_body_size)
        .await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    if let Ok(text) = std::str::from_utf8(&body_bytes) {
        debug!("📨 Request body:\n{}", text);
    } else {
        debug!("📨 Request body: <binary {} bytes>", body_bytes.len());
    }

    // 5a. 从请求体提取模型名，构建请求路径: 上游名/模型名
    let model_name = extract_model(&body_bytes);
    let upstream_name = state.upstream_name.as_ref();
    let audit_path = if !model_name.is_empty() {
        if !upstream_name.is_empty() {
            format!("{}/{}", upstream_name, model_name)
        } else {
            model_name
        }
    } else if !upstream_name.is_empty() {
        upstream_name.to_string()
    } else {
        path_and_query // 回退到原始 URL 路径
    };

    // 6. 敏感数据检测与替换
    let mut placeholder_map: Option<PlaceholderMap> = None;
    let mut sanitized_body: Option<String> = None;
    let mut original_body: Option<String> = None;
    let mut hits_for_audit: Option<Vec<crate::detector::Match>> = None;

    if let Ok(text) = std::str::from_utf8(&body_bytes) {
        let d = state.detector.read().await;
        let hits = d.detect(text);
        if !hits.is_empty() {
            info!("检测到敏感数据: {} 处", hits.len());
            for m in &hits {
                info!(
                    "  规则: {} | 策略: {:?} | 位置: {}..{}",
                    m.rule_id, m.strategy, m.start, m.end
                );
            }

            let (sanitized, map) = replacer::replace(text, &hits);
            debug!("替换后: {}", sanitized);

            original_body = Some(text.to_string());
            hits_for_audit = Some(hits);
            sanitized_body = Some(sanitized.clone());
            body_bytes = axum::body::Bytes::from(sanitized);
            placeholder_map = Some(map);
        }
    }

    // 7. 转发请求
    let body_vec = body_bytes.to_vec();
    let upstream_resp = state
        .forwarder
        .forward(method, &target_url, headers, body_vec, &state.api_key)
        .await
        .map_err(|e| {
            error!("Forward failed: {}", e);
            (StatusCode::BAD_GATEWAY, e.to_string())
        })?;

    // 8. 审计记录：在获知响应状态后写入 storage
    let status_code = upstream_resp.status().as_u16();
    if let (Some(ref storage), Some(ref map), Some(ref sani_body), Some(ref hits), Some(ref orig_body)) =
        (&state.storage, &placeholder_map, &sanitized_body, &hits_for_audit, &original_body)
    {
        for placeholder in map.placeholders() {
            if let Some(original) = map.get(placeholder) {
                let rule_id = placeholder
                    .strip_prefix("[[")
                    .and_then(|s| s.split_once('@'))
                    .map(|(id, _)| id)
                    .unwrap_or("unknown");
                let context = hits
                    .iter()
                    .find(|m| m.text == *original)
                    .map(|m| extract_context(orig_body, m.start, m.end, 80))
                    .unwrap_or_default();
                let _ = storage.record(
                    rule_id,
                    "placeholder",
                    placeholder,
                    original,
                    &context,
                    &audit_path,
                    sani_body,
                    status_code,
                    &tool_name,
                );
            }
        }
    }

    // 8b. 广播检测事件（无论 storage 是否启用）
    if let Some(ref tx) = state.event_tx {
        if let Some(ref map) = placeholder_map {
            for placeholder in map.placeholders() {
                let rule_id = placeholder
                    .strip_prefix("[[")
                    .and_then(|s| s.split_once('@'))
                    .map(|(id, _)| id)
                    .unwrap_or("unknown");
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;
                let _ = tx.send(DetectionEvent {
                    timestamp_ms: now_ms,
                    rule_id: rule_id.to_string(),
                    strategy: "placeholder".to_string(),
                    placeholder: placeholder.clone(),
                    request_path: audit_path.clone(),
                    response_status: status_code,
                    tool_name: tool_name.clone(),
                });
            }
        }
    }

    let status = upstream_resp.status();
    let resp_headers = upstream_resp.headers().clone();

    let is_stream = resp_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false);

    // 7. 流式 / 非流式响应处理
    if is_stream {
        info!("🌊 Streaming response detected, switching to SSE passthrough");
        let (status, resp_headers, body) = if let Some(map) = placeholder_map {
            info!("🔓 Streaming with placeholder restore ({} mapping(s))", map.len());
            stream::stream_response_with_restore(upstream_resp, map)
        } else {
            stream::stream_response(upstream_resp)
        };
        info!("<-- {} (streaming)", status);

        let mut response = Response::builder().status(status);
        for (key, value) in &resp_headers {
            if key == "transfer-encoding" {
                continue;
            }
            response = response.header(key, value);
        }

        return response.body(body).map_err(|e| {
            error!("Failed to build streaming response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        });
    }

    // 非流式：读取完整响应体
    let resp_bytes = upstream_resp.bytes().await.map_err(|e| {
        error!("Failed to read upstream response: {}", e);
        (StatusCode::BAD_GATEWAY, e.to_string())
    })?;

    let mut resp_text = String::new();
    let was_utf8 = match std::str::from_utf8(&resp_bytes) {
        Ok(text) => {
            resp_text = text.to_string();
            true
        }
        Err(_) => false,
    };

    if was_utf8 {
        if let Some(ref map) = placeholder_map {
            let restored = replacer::restore(&resp_text, map);
            info!("📩 Response (restored {} placeholders):\n{}", map.len(), restored);
            resp_text = restored;
        } else {
            debug!("📩 Response body:\n{}", resp_text);
        }
    } else {
        debug!("📩 Response body: <binary {} bytes>", resp_bytes.len());
    }

    info!("<-- {} ({} bytes)", status, resp_text.len());

    let mut response = Response::builder().status(status);
    for (key, value) in &resp_headers {
        if key == "transfer-encoding" {
            continue;
        }
        response = response.header(key, value);
    }

    response
        .body(Body::from(resp_text))
        .map_err(|e| {
            error!("Failed to build response: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

/// 从 HTTP 请求头中提取 User-Agent 原始值作为工具名。
fn extract_client_name(headers: &axum::http::HeaderMap) -> String {
    if let Some(ua) = headers.get("user-agent") {
        if let Ok(s) = ua.to_str() {
            let s = s.trim();
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    String::new()
}

/// 从 OpenAI 兼容格式的请求体中提取模型名。
fn extract_model(body: &[u8]) -> String {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(model) = json.get("model").and_then(|v| v.as_str()) {
            return model.to_string();
        }
    }
    String::new()
}

fn extract_context(text: &str, start: usize, end: usize, radius: usize) -> String {
    let mut ctx_start = start.saturating_sub(radius);
    let mut ctx_end = (end + radius).min(text.len());
    while ctx_start > 0 && !text.is_char_boundary(ctx_start) {
        ctx_start -= 1;
    }
    while ctx_end < text.len() && !text.is_char_boundary(ctx_end) {
        ctx_end += 1;
    }
    text[ctx_start..ctx_end].to_string()
}

/// 过滤 hop-by-hop 头，同时移除客户端的 Authorization（由 Aidaguard 统一注入）
fn forward_headers(headers: &HeaderMap) -> HeaderMap {
    let mut new_headers = HeaderMap::new();

    let skip = [
        "host",
        "authorization",
        "connection",
        "content-encoding",
        "content-length",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "transfer-encoding",
        "upgrade",
    ];

    for (key, value) in headers {
        let key_str = key.as_str().to_lowercase();
        if !skip.contains(&key_str.as_str()) {
            if let Ok(v) = HeaderValue::from_bytes(value.as_bytes()) {
                new_headers.insert(key, v);
            }
        }
    }

    new_headers
}
