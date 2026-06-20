use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{any, get, post},
    Json, Router,
};
use reqwest::Method;
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};

use aidaguard_core::config::Config;
use aidaguard_core::detector::Match;
use aidaguard_core::DetectionEngine;
use aidaguard_core::AuditStorage;
use aidaguard_core::replacer::{self, PlaceholderMap};
use aidaguard_storage::Storage;
use aidaguard_detector::AnalyzerEngine;
use crate::forwarder::Forwarder;
use crate::stream;
use crate::DetectionEvent;

/// 代理共享状态
#[derive(Clone)]
struct ProxyState {
    forwarder: Forwarder,
    api_key: Arc<String>,
    target_url: Arc<String>,
    upstream_name: Arc<String>,
    detector: Arc<RwLock<AnalyzerEngine>>,
    storage: Option<Arc<Storage>>,
    start_time: Instant,
    version: &'static str,
    max_body_size: usize,
    rules_dir: Arc<String>,
    presets: Arc<Vec<String>>,
    event_tx: Option<tokio::sync::broadcast::Sender<DetectionEvent>>,
}

/// 启动代理服务器（便捷包装，用于 CLI 独立运行）。
/// 内部创建 Detector / Storage 并永不主动关闭。
pub async fn start(mut config: Config) -> Result<(), anyhow::Error> {
    let upstream_name = resolve_upstream_name(&mut config);
    let presets = config.rule_presets();
    let engine = AnalyzerEngine::builder()
        .with_all_pattern_recognizers()
        .with_config_rules(&config)
        .with_nlp_config(&config.nlp)
        .with_min_confidence(0.3)
        .build()?;
    let detector = Arc::new(RwLock::new(engine));
    let storage = open_storage(&config);
    start_with_state(
        config, presets, detector, storage, None,
        std::future::pending(), upstream_name,
    ).await
}

/// 从 config 解析默认上游的名称，同时填充 target_url 和 api_key。
fn resolve_upstream_name(config: &mut Config) -> String {
    if config.target_url.is_empty() {
        if let Some(up) = config.upstreams.iter().find(|u| u.default) {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            return up.name.clone();
        }
        if let Some(up) = config.upstreams.first() {
            config.target_url = up.url.clone();
            if let Some(ref key) = up.api_key {
                config.api_key = key.clone();
            }
            return up.name.clone();
        }
        String::new()
    } else {
        config.upstreams.iter()
            .find(|u| u.url == config.target_url)
            .map(|u| u.name.clone())
            .unwrap_or_default()
    }
}

/// 启动代理服务器，接受外部管理的 Detector / Storage 及关闭信号。
pub async fn start_with_state<F>(
    config: Config,
    presets: Vec<String>,
    detector: Arc<RwLock<AnalyzerEngine>>,
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

    let mut upstream_mgr = aidaguard_upstream::UpstreamManager::new();
    let builtin_count = upstream_mgr.load_builtins();
    info!("Loaded {} built-in LLM providers", builtin_count);

    // Look up rate_limit_qps from matching upstream config
    let rate_limit_qps = config.upstreams.iter()
        .find(|u| u.url == config.target_url)
        .map(|u| u.rate_limit_qps)
        .unwrap_or(0);

    let forwarder = if let Some(provider) = upstream_mgr.find_by_endpoint(&config.target_url) {
        info!(
            "Matched provider '{}' for target URL, using {:?} auth",
            provider.id, provider.auth
        );
        Forwarder::from_upstream(provider, api_key.clone(), 300, rate_limit_qps)?
    } else {
        info!("No built-in provider matched, using default Bearer auth");
        let f = Forwarder::new()?.with_bearer_auth(api_key.clone());
        if rate_limit_qps > 0 { f.with_rate_limit(rate_limit_qps) } else { f }
    };

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
        version: aidaguard_core::VERSION,
        max_body_size,
        rules_dir: rules_dir.clone(),
        presets: Arc::new(presets),
        event_tx,
    };

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/reload", post(reload_rules))
        .route("/", any(proxy_handler))
        .route("/*path", any(proxy_handler))
        .layer(cors)
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
                warn!("\u{65e0}\u{6cd5}\u{521b}\u{5efa}\u{5b58}\u{50a8}\u{76ee}\u{5f55}: {}", e);
                return None;
            }
        }
        // Require explicit key when storage is enabled; fail hard if missing
        let enc_key = config.storage.encryption_key.as_deref().unwrap_or_else(|| {
            warn!("Storage enabled but encryption_key is not set, using default key");
            "aidaguard-internal-key"
        });
        match Storage::open(std::path::Path::new(&config.storage.db_path), enc_key) {
            Ok(s) => {
                info!("Storage enabled: {}", config.storage.db_path);
                Some(Arc::new(s))
            }
            Err(e) => {
                warn!("Storage \u{6253}\u{5f00}\u{5931}\u{8d25}: {}", e);
                None
            }
        }
    } else {
        None
    }
}

/// GET /health
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

/// POST /reload
async fn reload_rules(
    axum::extract::State(state): axum::extract::State<ProxyState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let rules_path = std::path::Path::new(state.rules_dir.as_str());
    let mut engine = state.detector.write().await;
    match engine.reload_presets(rules_path, &state.presets) {
        Ok(count) => {
            info!("\u{89c4}\u{5219}\u{624b}\u{52a8}\u{91cd}\u{8f7d}\u{5b8c}\u{6210}: {} \u{6761}", count);
            Ok(Json(serde_json::json!({
                "status": "ok",
                "rules_count": count,
                "message": format!("\u{5df2}\u{52a0}\u{8f7d} {} \u{6761}\u{89c4}\u{5219}", count),
            })))
        }
        Err(e) => {
            error!("\u{89c4}\u{5219}\u{91cd}\u{8f7d}\u{5931}\u{8d25}: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"status": "error", "message": e.to_string()})),
            ))
        }
    }
}

/// 代理转发 handler：检测 \u{2192} 替换 \u{2192} 转发 \u{2192} 还原
async fn proxy_handler(
    axum::extract::State(state): axum::extract::State<ProxyState>,
    req: Request,
) -> Result<Response, (StatusCode, String)> {
    let tool_name = extract_client_name(req.headers());
    let path_and_query = req.uri().path_and_query()
        .map(|pq| pq.as_str()).unwrap_or("").to_string();
    let target_url = build_target_url(&state.target_url, &path_and_query);
    let method: Method = req.method().clone();
    let headers = forward_headers(req.headers());

    info!("--> {} {}", req.method(), target_url);

    check_body_size(req.headers(), state.max_body_size)?;

    let body_bytes = read_body(req, state.max_body_size).await?;
    let audit_path = build_audit_path(&state.upstream_name, &body_bytes, &path_and_query);
    let detection = detect_and_replace(&state.detector, &body_bytes).await;

    let placeholder_map = detection.placeholder_map;
    let sanitized_body = detection.sanitized_body;
    let original_body = detection.original_body;
    let hits_for_audit = detection.hits_for_audit;
    let body_vec = detection.bytes.to_vec();

    let upstream_resp = state
        .forwarder
        .forward(method, &target_url, headers, body_vec, &state.api_key)
        .await
        .map_err(|e| {
            error!("Forward failed: {}", e);
            (StatusCode::BAD_GATEWAY, e.to_string())
        })?;

    let status_code = upstream_resp.status().as_u16();

    write_audit_records(
        &state.storage, &state.detector, &placeholder_map, &hits_for_audit,
        &sanitized_body, &original_body, &audit_path, status_code, &tool_name,
    ).await;

    broadcast_events(&state.event_tx, &placeholder_map, &hits_for_audit, status_code, &audit_path, &tool_name);

    if is_streaming_response(upstream_resp.headers()) {
        return handle_streaming_response(upstream_resp, placeholder_map);
    }

    handle_non_streaming_response(upstream_resp, placeholder_map).await
}

// ── helper functions extracted from proxy_handler ──

fn build_target_url(base: &str, path_and_query: &str) -> String {
    if path_and_query.is_empty() || path_and_query == "/" {
        base.to_string()
    } else {
        format!("{}{}", base, path_and_query)
    }
}

fn build_audit_path(upstream_name: &str, body_bytes: &[u8], path_and_query: &str) -> String {
    let model_name = extract_model(body_bytes);
    if !model_name.is_empty() {
        if !upstream_name.is_empty() {
            format!("{}/{}", upstream_name, model_name)
        } else {
            model_name
        }
    } else if !upstream_name.is_empty() {
        upstream_name.to_string()
    } else {
        path_and_query.to_string()
    }
}

fn check_body_size(headers: &HeaderMap, max_body_size: usize) -> Result<(), (StatusCode, String)> {
    let content_length = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length > max_body_size {
        warn!(
            "Request body too large: {} bytes (limit: {} MB)",
            content_length, max_body_size / (1024 * 1024)
        );
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("Request body cannot exceed {} MB", max_body_size / (1024 * 1024)),
        ));
    }
    Ok(())
}

async fn read_body(req: Request, max_body_size: usize) -> Result<axum::body::Bytes, (StatusCode, String)> {
    axum::body::to_bytes(req.into_body(), max_body_size).await.map_err(|e| {
        error!("Failed to read request body: {}", e);
        (StatusCode::BAD_REQUEST, e.to_string())
    })
}

struct DetectionResult {
    placeholder_map: Option<PlaceholderMap>,
    sanitized_body: Option<String>,
    original_body: Option<String>,
    hits_for_audit: Option<Vec<Match>>,
    bytes: axum::body::Bytes,
}

async fn detect_and_replace(
    detector: &Arc<RwLock<AnalyzerEngine>>,
    body_bytes: &[u8],
) -> DetectionResult {
    let Ok(text) = std::str::from_utf8(body_bytes) else {
        return DetectionResult {
            placeholder_map: None,
            sanitized_body: None,
            original_body: None,
            hits_for_audit: None,
            bytes: axum::body::Bytes::from(body_bytes.to_vec()),
        };
    };

    let d = detector.read().await;
    let all_hits = d.detect(text);

    if all_hits.is_empty() {
        return DetectionResult {
            placeholder_map: None,
            sanitized_body: None,
            original_body: None,
            hits_for_audit: None,
            bytes: axum::body::Bytes::from(body_bytes.to_vec()),
        };
    }

    info!("\u{68c0}\u{6d4b}\u{5230}\u{654f}\u{611f}\u{6570}\u{636e}: {} \u{5904}", all_hits.len());
    for m in &all_hits {
        info!(
            "  Rule: {} | Strategy: {:?} | Mode: {:?} | Pos: {}..{}",
            m.rule_id, m.strategy, m.mode, m.start, m.end
        );
    }

    let filter_hits: Vec<Match> = all_hits.iter()
        .filter(|m| m.mode == aidaguard_core::detector::Mode::Filter)
        .cloned()
        .collect();

    let original_body = Some(text.to_string());

    if filter_hits.is_empty() {
        return DetectionResult {
            placeholder_map: None,
            sanitized_body: Some(text.to_string()),
            original_body,
            hits_for_audit: Some(all_hits),
            bytes: axum::body::Bytes::from(body_bytes.to_vec()),
        };
    }

    let (sanitized, map) = replacer::replace(text, &filter_hits);
    debug!("After replacement: {}", sanitized);
    let sanitized_bytes = axum::body::Bytes::from(sanitized.clone());

    DetectionResult {
        placeholder_map: Some(map),
        sanitized_body: Some(sanitized.clone()),
        original_body,
        hits_for_audit: Some(all_hits),
        bytes: sanitized_bytes,
    }
}



fn is_streaming_response(headers: &HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false)
}

fn handle_streaming_response(
    upstream_resp: reqwest::Response,
    placeholder_map: Option<PlaceholderMap>,
) -> Result<Response, (StatusCode, String)> {
    info!("Streaming response detected, switching to SSE passthrough");
    let (status, resp_headers, body) = if let Some(map) = placeholder_map {
        info!("Streaming with placeholder restore ({} mapping(s))", map.len());
        stream::stream_response_with_restore(upstream_resp, map)
    } else {
        stream::stream_response(upstream_resp)
    };
    info!("<-- {} (streaming)", status);

    let mut response = Response::builder().status(status);
    for (key, value) in &resp_headers {
        response = response.header(key, value);
    }
    response.body(body).map_err(|e| {
        error!("Failed to build streaming response: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })
}

async fn handle_non_streaming_response(
    upstream_resp: reqwest::Response,
    placeholder_map: Option<PlaceholderMap>,
) -> Result<Response, (StatusCode, String)> {
    let status = upstream_resp.status();
    let resp_headers = upstream_resp.headers().clone();

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
            info!("Response (restored {} placeholders)", map.len());
            resp_text = restored;
        }
    }

    info!("<-- {} ({} bytes)", status, resp_text.len());

    let mut response = Response::builder().status(status);
    for (key, value) in &resp_headers {
        if key == "transfer-encoding" {
            continue;
        }
        response = response.header(key, value);
    }

    response.body(Body::from(resp_text)).map_err(|e| {
        error!("Failed to build response: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })
}

async fn write_audit_records(
    storage: &Option<Arc<Storage>>,
    detector: &Arc<RwLock<AnalyzerEngine>>,
    placeholder_map: &Option<PlaceholderMap>,
    hits_for_audit: &Option<Vec<Match>>,
    sanitized_body: &Option<String>,
    original_body: &Option<String>,
    audit_path: &str,
    status_code: u16,
    tool_name: &str,
) {
    let (Some(ref storage), Some(ref sani_body), Some(ref hits), Some(ref orig_body)) =
        (storage, sanitized_body, hits_for_audit, original_body) else { return };

    let detector = detector.read().await;

    if let Some(ref map) = placeholder_map {
        for placeholder in map.placeholders() {
            let Some(original) = map.get(placeholder) else { continue };
            let rule_id = placeholder
                .strip_prefix("[[")
                .and_then(|s| s.split_once('@'))
                .map(|(id, _)| id)
                .unwrap_or("unknown");
            let rule_name = detector.rule_name(rule_id).unwrap_or(rule_id);
            let context = hits.iter()
                .find(|m| m.text == *original)
                .map(|m| extract_context(orig_body, m.start, m.end, 80))
                .unwrap_or_default();
            let _ = storage.record(rule_id, rule_name, "placeholder", placeholder, original,
                &context, audit_path, sani_body, status_code, tool_name);
        }
    }

    for m in hits {
        if m.mode == aidaguard_core::detector::Mode::Detect {
            let rule_name = detector.rule_name(&m.rule_id).unwrap_or(&m.rule_id);
            let context = extract_context(orig_body, m.start, m.end, 80);
            let _ = storage.record(&m.rule_id, rule_name, "detect", "", &m.text,
                &context, audit_path, sani_body, status_code, tool_name);
        }
    }
}

fn broadcast_events(
    event_tx: &Option<tokio::sync::broadcast::Sender<DetectionEvent>>,
    placeholder_map: &Option<PlaceholderMap>,
    hits_for_audit: &Option<Vec<Match>>,
    status_code: u16,
    audit_path: &str,
    tool_name: &str,
) {
    let Some(ref tx) = event_tx else { return };

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    if let Some(ref map) = placeholder_map {
        for placeholder in map.placeholders() {
            let rule_id = placeholder
                .strip_prefix("[[")
                .and_then(|s| s.split_once('@'))
                .map(|(id, _)| id)
                .unwrap_or("unknown");
            let _ = tx.send(DetectionEvent {
                timestamp_ms: now_ms,
                rule_id: rule_id.to_string(),
                strategy: "placeholder".to_string(),
                placeholder: placeholder.clone(),
                request_path: audit_path.to_string(),
                response_status: status_code,
                tool_name: tool_name.to_string(),
            });
        }
    }

    if let Some(ref hits) = hits_for_audit {
        for m in hits.iter().filter(|m| m.mode == aidaguard_core::detector::Mode::Detect) {
            let _ = tx.send(DetectionEvent {
                timestamp_ms: now_ms,
                rule_id: m.rule_id.clone(),
                strategy: "detect".to_string(),
                placeholder: String::new(),
                request_path: audit_path.to_string(),
                response_status: status_code,
                tool_name: tool_name.to_string(),
            });
        }
    }
}

/// 从 HTTP 请求头中提取 User-Agent 原始值作为工具名。
fn extract_client_name(headers: &HeaderMap) -> String {
    headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// 从 OpenAI 兼容格式的请求体中提取模型名。
fn extract_model(body: &[u8]) -> String {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|json| json.get("model").and_then(|v| v.as_str().map(String::from)))
        .unwrap_or_default()
}

fn extract_context(text: &str, start: usize, end: usize, radius: usize) -> String {
    let ctx_start = start.saturating_sub(radius);
    let ctx_end = (end + radius).min(text.len());
    // Use floor_char_boundary for safe char-boundary snapping
    let adj_start = ctx_start.max(0);
    let adj_end = if ctx_end < text.len() {
        text.floor_char_boundary(ctx_end)
    } else {
        ctx_end
    };
    text[adj_start..adj_end].to_string()
}

/// 过滤 hop-by-hop 头，同时移除客户端的 Authorization（由 Aidaguard 统一注入）。
/// 直接 clone HeaderValue 而非重新解析 bytes。
fn forward_headers(headers: &HeaderMap) -> HeaderMap {
    let mut new_headers = HeaderMap::new();

    let skip = [
        "host", "authorization", "connection", "content-encoding",
        "content-length", "keep-alive", "proxy-authenticate",
        "proxy-authorization", "te", "trailers", "transfer-encoding", "upgrade",
    ];

    for (key, value) in headers {
        if !skip.contains(&key.as_str().to_lowercase().as_str()) {
            new_headers.insert(key, value.clone());
        }
    }

    new_headers
}
