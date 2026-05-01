use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
    routing::any,
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::detector::Detector;
use crate::replacer::{self, PlaceholderMap};
use super::stream;

/// 默认目标 API 地址（可通过 AIDAGUARD_TARGET_URL 环境变量覆盖）
const DEFAULT_TARGET_URL: &str = "https://qianfan.baidubce.com/v2/coding";

/// 本地监听端口
pub const PROXY_PORT: u16 = 19000;

/// 代理共享状态
#[derive(Clone)]
struct ProxyState {
    client: Client,
    api_key: Arc<String>,
    target_url: Arc<String>,
    detector: Arc<RwLock<Detector>>,
}

/// 启动代理服务器
pub async fn start() -> Result<()> {
    // 从环境变量读取 API Key，启动时必须提供
    let api_key = std::env::var("AIDAGUARD_API_KEY")
        .context("环境变量 AIDAGUARD_API_KEY 未设置，请先执行：export AIDAGUARD_API_KEY=你的Key")?;

    let auth_value = if api_key.starts_with("Bearer ") {
        api_key
    } else {
        format!("Bearer {}", api_key)
    };

    info!("API Key 已加载（前8位：{}...）", &auth_value[..8.min(auth_value.len())]);

    let target_url = std::env::var("AIDAGUARD_TARGET_URL")
        .unwrap_or_else(|_| DEFAULT_TARGET_URL.to_string());

    info!("Target URL: {}", target_url);

    // 加载检测规则
    let mut detector = Detector::new();
    let rules_dir = std::env::var("AIDAGUARD_RULES_DIR")
        .unwrap_or_else(|_| "./rules".to_string());
    let rules_path = std::path::Path::new(&rules_dir);
    if rules_path.exists() {
        detector.load_from_dir(rules_path)?;
    } else {
        warn!("规则目录不存在: {}", rules_dir);
    }

    let detector = Arc::new(RwLock::new(detector));

    let state = ProxyState {
        client: Client::builder().use_rustls_tls().build()?,
        api_key: Arc::new(auth_value),
        target_url: Arc::new(target_url),
        detector: detector.clone(),
    };

    let app = Router::new()
        .route("/", any(handle))
        .route("/*path", any(handle))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", PROXY_PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Aidaguard proxy listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

/// 处理所有进来的请求
async fn handle(
    axum::extract::State(state): axum::extract::State<ProxyState>,
    req: Request,
) -> Result<Response, (StatusCode, String)> {
    // 1. 拼接目标 URL
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("");

    let target_url = if path_and_query == "/" || path_and_query.is_empty() {
        state.target_url.to_string()
    } else {
        format!("{}{}", state.target_url, path_and_query)
    };

    info!("--> {} {}", req.method(), target_url);

    // 2. 处理请求头
    let method = req.method().clone();
    let mut headers = forward_headers(req.headers());

    // 注入统一的 API Key
    match HeaderValue::from_str(&state.api_key) {
        Ok(v) => {
            headers.insert("authorization", v);
        }
        Err(e) => {
            error!("API Key 格式无效: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "API Key 格式无效".to_string()));
        }
    }

    // 3. 读取请求体
    let mut body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .map_err(|e| {
            error!("Failed to read request body: {}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    // 4. 打印请求体内容（debug 级别，避免干扰检测输出）
    if let Ok(text) = std::str::from_utf8(&body_bytes) {
        debug!("📨 Request body:\n{}", text);
    } else {
        debug!("📨 Request body: <binary {} bytes>", body_bytes.len());
    }

    // 4.5. 敏感数据检测与替换
    let mut placeholder_map: Option<PlaceholderMap> = None;

    if let Ok(text) = std::str::from_utf8(&body_bytes) {
        let d = state.detector.read().await;
        let hits = d.detect(text);
        if !hits.is_empty() {
            warn!("══════════════════════════════════════════");
            warn!("🔍 检测到敏感数据: {} 处", hits.len());
            for m in &hits {
                warn!("  规则: {} | 内容: \"{}\" | 策略: {:?}", m.rule_id, m.text, m.strategy);
            }

            let (sanitized, map) = replacer::replace(text, &hits);
            warn!("📝 替换后: {}", sanitized);
            warn!("══════════════════════════════════════════");

            body_bytes = axum::body::Bytes::from(sanitized);
            placeholder_map = Some(map);
        }
    }

    // 5. 转发请求
    let upstream_resp = state
        .client
        .request(method, &target_url)
        .headers(headers)
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| {
            error!("Upstream request failed: {}", e);
            (StatusCode::BAD_GATEWAY, e.to_string())
        })?;

    // 6. 判断是否为流式响应，处理返回
    let status = upstream_resp.status();
    let resp_headers = upstream_resp.headers().clone();

    let is_stream = resp_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false);

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

    // 7. 处理响应体：还原占位符 + 日志
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

    // 8. 构造返回给客户端的响应
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
