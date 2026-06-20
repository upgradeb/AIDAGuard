use axum::body::{Body, Bytes};
use axum::http::{HeaderMap, StatusCode};
use futures::stream::StreamExt;
use reqwest::Response;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

use aidaguard_core::replacer::{self, PlaceholderMap};

/// 纯透传，不做还原
pub fn stream_response(upstream_resp: Response) -> (StatusCode, HeaderMap, Body) {
    let status = upstream_resp.status();
    let headers = filter_transport_headers(upstream_resp.headers());

    let stream = upstream_resp.bytes_stream().map(|result| match result {
        Ok(bytes) => {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                debug!("\u{1f4e9} SSE chunk: {}", text.trim());
            }
            Ok::<_, axum::Error>(Bytes::from(bytes))
        }
        Err(e) => {
            error!("Stream error: {}", e);
            Err(axum::Error::new(e))
        }
    });

    (status, headers, Body::from_stream(stream))
}

/// 流式透传 + 占位符还原
///
/// 解析每条 SSE JSON → 提取 `content`、`reasoning_content` 或 `tool_calls[0].function.arguments` →
/// 累积到文本缓冲区（content 和 reasoning 分别缓冲）→ 前缀匹配安全分割 →
/// 还原完整占位符 → 修改 JSON 对应字段 → 重新序列化转发。
pub fn stream_response_with_restore(
    upstream_resp: Response,
    map: PlaceholderMap,
) -> (StatusCode, HeaderMap, Body) {
    let status = upstream_resp.status();
    let headers = filter_transport_headers(upstream_resp.headers());

    let map = Arc::new(map);
    let text_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let reasoning_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let map_for_flush = map.clone();
    let text_buf_for_flush = text_buf.clone();
    let reasoning_buf_for_flush = reasoning_buf.clone();

    let stream = upstream_resp
        .bytes_stream()
        .map(move |result| match result {
            Ok(bytes) => {
                let chunk_str = std::str::from_utf8(&bytes).unwrap_or("");
                debug!("\u{1f4e9} SSE chunk: {}", &chunk_str[..chunk_str.len().min(120)]);
                let output = process_sse_chunk(chunk_str, &text_buf, &reasoning_buf, &map);
                Ok::<_, axum::Error>(Bytes::from(output))
            }
            Err(e) => {
                error!("Stream error: {}", e);
                Err(axum::Error::new(e))
            }
        })
        .chain(futures::stream::once(async move {
            let mut flush_output = String::new();

            // flush content buffer
            let remaining = {
                let buf = text_buf_for_flush.lock().unwrap();
                buf.clone()
            };
            if !remaining.is_empty() {
                let restored = replacer::restore(&remaining, &map_for_flush);
                debug!("stream flush content {} bytes \u{2192} {} bytes", remaining.len(), restored.len());
                let delta = serde_json::json!({
                    "choices": [{"delta": {"content": restored}, "index": 0}]
                });
                flush_output.push_str(&format!("data: {}\n\n", serde_json::to_string(&delta).unwrap()));
            }

            // flush reasoning_content buffer
            let remaining = {
                let buf = reasoning_buf_for_flush.lock().unwrap();
                buf.clone()
            };
            if !remaining.is_empty() {
                let restored = replacer::restore(&remaining, &map_for_flush);
                debug!("stream flush reasoning {} bytes \u{2192} {} bytes", remaining.len(), restored.len());
                let delta = serde_json::json!({
                    "choices": [{"delta": {"reasoning_content": restored}, "index": 0}]
                });
                flush_output.push_str(&format!("data: {}\n\n", serde_json::to_string(&delta).unwrap()));
            }

            Ok::<_, axum::Error>(Bytes::from(flush_output))
        }));

    (status, headers, Body::from_stream(stream))
}

/// 使用行级 SSE 解析，逐行读取并累积完整事件，避免 split("\\n\\n")
/// 拆断 JSON payload 中的嵌入空行。
fn process_sse_chunk(
    chunk_str: &str,
    text_buf: &Mutex<String>,
    reasoning_buf: &Mutex<String>,
    map: &PlaceholderMap,
) -> String {
    let mut output = String::new();
    let mut current_event = String::new();

    for line in chunk_str.lines() {
        if line.is_empty() {
            // 空行 = SSE 事件分隔符
            if !current_event.is_empty() {
                let processed = process_one_message(&current_event, text_buf, reasoning_buf, map);
                output.push_str(&processed);
                current_event.clear();
            }
        } else {
            if !current_event.is_empty() {
                current_event.push('\n');
            }
            current_event.push_str(line);
        }
    }

    // 处理没有尾随 \\n\\n 的剩余数据
    if !current_event.is_empty() {
        let processed = process_one_message(&current_event, text_buf, reasoning_buf, map);
        output.push_str(&processed);
    }

    output
}

/// 处理单条 SSE 消息（已完成的事件，不含尾部 \\n\\n）
fn process_one_message(
    msg: &str,
    text_buf: &Mutex<String>,
    reasoning_buf: &Mutex<String>,
    map: &PlaceholderMap,
) -> String {
    let trimmed = msg.trim();

    if trimmed == "data: [DONE]" {
        return format!("{}\n\n", msg);
    }

    let json_str = match trimmed.strip_prefix("data: ") {
        Some(s) => s,
        None => return format!("{}\n\n", msg),
    };

    let mut value: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return format!("{}\n\n", msg),
    };

    // 判断文本字段类型，选择对应缓冲区
    let text_info = extract_text_info(&value);

    let (field_path, text_to_add, buf) = match text_info {
        Some(TextKind::Content { path, text }) => (path, text.to_string(), text_buf),
        Some(TextKind::Reasoning { path, text }) => (path, text.to_string(), reasoning_buf),
        Some(TextKind::ToolCall { path, text }) => (path, text.to_string(), text_buf),
        None => return format!("data: {}\n\n", json_str),
    };

    let mut buf = buf.lock().unwrap();
    buf.push_str(&text_to_add);

    let safe_len = find_safe_len(&buf, map);

    if safe_len == 0 {
        debug!("text buffer all unsafe, holding {} bytes", buf.len());
        return String::new();
    }

    let safe_text = buf[..safe_len].to_string();
    *buf = buf[safe_len..].to_string();

    let restored = replacer::restore(&safe_text, map);

    if let Some(v) = value.pointer_mut(&field_path) {
        *v = Value::String(restored);
    }

    let new_json = serde_json::to_string(&value).unwrap();
    format!("data: {}\n\n", new_json)
}

/// 文本字段类型
enum TextKind<'a> {
    Content { path: String, text: &'a str },
    Reasoning { path: String, text: &'a str },
    ToolCall { path: String, text: &'a str },
}

/// 从 delta JSON 中提取文本内容，按优先级：content > reasoning_content > tool_calls arguments
fn extract_text_info(value: &Value) -> Option<TextKind<'_>> {
    if let Some(s) = value
        .pointer("/choices/0/delta/content")
        .and_then(|v| v.as_str())
    {
        if !s.is_empty() {
            return Some(TextKind::Content {
                path: "/choices/0/delta/content".to_string(),
                text: s,
            });
        }
    }

    if let Some(s) = value
        .pointer("/choices/0/delta/reasoning_content")
        .and_then(|v| v.as_str())
    {
        if !s.is_empty() {
            return Some(TextKind::Reasoning {
                path: "/choices/0/delta/reasoning_content".to_string(),
                text: s,
            });
        }
    }

    if let Some(s) = value
        .pointer("/choices/0/delta/tool_calls/0/function/arguments")
        .and_then(|v| v.as_str())
    {
        if !s.is_empty() {
            return Some(TextKind::ToolCall {
                path: "/choices/0/delta/tool_calls/0/function/arguments".to_string(),
                text: s,
            });
        }
    }

    None
}

/// Pre-computed prefix set for efficient safe-length lookup.
struct PlaceholderPrefixIndex {
    prefixes: Vec<String>,
}

impl PlaceholderPrefixIndex {
    fn from_map(map: &PlaceholderMap) -> Self {
        let mut prefixes: HashSet<String> = HashSet::new();
        for placeholder in map.placeholders() {
            for prefix_len in 1..placeholder.len() {
                prefixes.insert(placeholder[..prefix_len].to_string());
            }
        }
        // Sort by length descending so longest prefix is tried first
        let mut sorted: Vec<String> = prefixes.into_iter().collect();
        sorted.sort_by(|a, b| b.len().cmp(&a.len()));
        Self { prefixes: sorted }
    }

    /// Return the split offset: everything before this point is safe to forward.
    fn split_offset(&self, text: &str) -> usize {
        let mut keep_from = text.len();
        for prefix in &self.prefixes {
            if let Some(pos) = text.rfind(prefix.as_str()) {
                if pos < keep_from {
                    keep_from = pos;
                    // Short-circuit: can't go lower than 0
                    if keep_from == 0 {
                        break;
                    }
                }
            }
        }
        keep_from
    }
}

/// 在文本中寻找安全分割点：返回可安全还原并转发的字节位置。
/// 使用预计算的前缀索引，避免热路径上的 O(n\xc2\xb2) 扫描。
pub fn find_safe_len(text: &str, map: &PlaceholderMap) -> usize {
    let index = PlaceholderPrefixIndex::from_map(map);
    index.split_offset(text)
}

/// 过滤传输层 header，移除 hop-by-hop 头。
/// 只克隆需要的 header，避免全量 HeaderMap::clone()。
fn filter_transport_headers(headers: &HeaderMap) -> HeaderMap {
    let skip: [&str; 9] = [
        "host",
        "transfer-encoding",
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailers",
        "upgrade",
    ];

    let mut filtered = HeaderMap::new();
    for (key, value) in headers {
        let key_str = key.as_str().to_lowercase();
        if !skip.contains(&key_str.as_str()) {
            filtered.insert(key, value.clone());
        }
    }
    filtered
}
