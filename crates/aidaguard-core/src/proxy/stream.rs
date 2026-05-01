use axum::body::{Body, Bytes};
use axum::http::{HeaderMap, StatusCode};
use futures::stream::StreamExt;
use reqwest::Response;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

use crate::replacer::{self, PlaceholderMap};

/// 纯透传，不做还原
pub fn stream_response(upstream_resp: Response) -> (StatusCode, HeaderMap, Body) {
    let status = upstream_resp.status();
    let headers = upstream_resp.headers().clone();

    let stream = upstream_resp.bytes_stream().map(|result| match result {
        Ok(bytes) => {
            if let Ok(text) = std::str::from_utf8(&bytes) {
                debug!("📩 SSE chunk: {}", text.trim());
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
    let headers = upstream_resp.headers().clone();

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
                debug!("📩 SSE chunk: {}", &chunk_str[..chunk_str.len().min(120)]);
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
                debug!("stream flush content {} bytes → {} bytes", remaining.len(), restored.len());
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
                debug!("stream flush reasoning {} bytes → {} bytes", remaining.len(), restored.len());
                let delta = serde_json::json!({
                    "choices": [{"delta": {"reasoning_content": restored}, "index": 0}]
                });
                flush_output.push_str(&format!("data: {}\n\n", serde_json::to_string(&delta).unwrap()));
            }

            Ok::<_, axum::Error>(Bytes::from(flush_output))
        }));

    (status, headers, Body::from_stream(stream))
}

/// 处理单个 SSE chunk（可能包含多条消息）
fn process_sse_chunk(
    chunk_str: &str,
    text_buf: &Mutex<String>,
    reasoning_buf: &Mutex<String>,
    map: &PlaceholderMap,
) -> String {
    let mut output = String::new();
    let messages: Vec<&str> = chunk_str.split("\n\n").collect();

    for (i, msg) in messages.iter().enumerate() {
        if msg.is_empty() {
            continue;
        }
        let msg_with_nl = if i < messages.len() - 1 {
            format!("{}\n\n", msg)
        } else if chunk_str.ends_with("\n\n") {
            format!("{}\n\n", msg)
        } else {
            msg.to_string()
        };

        let processed = process_one_message(&msg_with_nl, text_buf, reasoning_buf, map);
        output.push_str(&processed);
    }

    output
}

/// 处理单条 SSE 消息
fn process_one_message(
    msg: &str,
    text_buf: &Mutex<String>,
    reasoning_buf: &Mutex<String>,
    map: &PlaceholderMap,
) -> String {
    let trimmed = msg.trim();

    if trimmed == "data: [DONE]" {
        return msg.to_string();
    }

    let json_str = match trimmed.strip_prefix("data: ") {
        Some(s) => s,
        None => return msg.to_string(),
    };

    let mut value: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return msg.to_string(),
    };

    // 判断文本字段类型，选择对应缓冲区
    let text_info = extract_text_info(&value);

    let (field_path, text_to_add, buf) = match text_info {
        Some(TextKind::Content { path, text }) => (path, text.to_string(), text_buf),
        Some(TextKind::Reasoning { path, text }) => (path, text.to_string(), reasoning_buf),
        Some(TextKind::ToolCall { path, text }) => (path, text.to_string(), text_buf),
        None => return msg.to_string(),
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

    if msg.ends_with("\n\n") {
        format!("data: {}\n\n", new_json)
    } else {
        format!("data: {}", new_json)
    }
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

/// 在文本中寻找安全分割点：返回可安全还原并转发的字节位置。
fn find_safe_len(text: &str, map: &PlaceholderMap) -> usize {
    let mut keep_from = text.len();

    for placeholder in map.placeholders() {
        for prefix_len in 1..placeholder.len() {
            let prefix = &placeholder[..prefix_len];
            if text.ends_with(prefix) {
                let start = text.len() - prefix_len;
                if start < keep_from {
                    keep_from = start;
                }
            }
        }
    }

    keep_from
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replacer::PlaceholderMap;

    fn make_map() -> PlaceholderMap {
        let mut map = PlaceholderMap::new();
        map.insert("13899991234", "PHONE_CN");
        map
    }

    #[test]
    fn test_find_safe_len_no_placeholder() {
        let map = make_map();
        assert_eq!(find_safe_len("普通文本", &map), "普通文本".len());
    }

    #[test]
    fn test_find_safe_len_partial_prefix() {
        let map = make_map();
        let text = "我的手机号 [[PHONE_CN@";
        let safe = find_safe_len(text, &map);
        let trailing = &text[safe..];
        assert!(trailing.starts_with("[[PHONE_CN@"));
    }

    #[test]
    fn test_find_safe_len_complete_placeholder() {
        let map = make_map();
        let placeholder = map.placeholders().next().unwrap().clone();
        let text = format!("我的手机号 {}", placeholder);
        assert_eq!(find_safe_len(&text, &map), text.len());
    }

    #[test]
    fn test_find_safe_len_complete_then_incomplete() {
        let map = make_map();
        let placeholder = map.placeholders().next().unwrap().clone();
        let text = format!("我的手机号 {} 另一个 [", placeholder);
        let safe = find_safe_len(&text, &map);
        assert_eq!(&text[safe..], "[");
    }

    #[test]
    fn test_find_safe_len_single_bracket() {
        let map = make_map();
        let text = "text [";
        let safe = find_safe_len(text, &map);
        assert_eq!(&text[safe..], "[");
    }
}
