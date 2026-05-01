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
/// 解析每条 SSE JSON → 提取 `content` 或 `tool_calls[0].function.arguments` →
/// 累积到文本缓冲区 → 在纯文本上做前缀匹配安全分割 →
/// 还原完整占位符 → 修改 JSON 对应字段 → 重新序列化转发。
///
/// 无文本内容的 delta（reasoning_content 等）原样透传。
pub fn stream_response_with_restore(
    upstream_resp: Response,
    map: PlaceholderMap,
) -> (StatusCode, HeaderMap, Body) {
    let status = upstream_resp.status();
    let headers = upstream_resp.headers().clone();

    let map = Arc::new(map);
    let text_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    let map_for_flush = map.clone();
    let text_buf_for_flush = text_buf.clone();

    let stream = upstream_resp
        .bytes_stream()
        .map(move |result| match result {
            Ok(bytes) => {
                let chunk_str = std::str::from_utf8(&bytes).unwrap_or("");
                debug!("📩 SSE chunk: {}", &chunk_str[..chunk_str.len().min(120)]);
                let output = process_sse_chunk(chunk_str, &text_buf, &map);
                Ok::<_, axum::Error>(Bytes::from(output))
            }
            Err(e) => {
                error!("Stream error: {}", e);
                Err(axum::Error::new(e))
            }
        })
        .chain(futures::stream::once(async move {
            let remaining = {
                let buf = text_buf_for_flush.lock().unwrap();
                buf.clone()
            };
            if remaining.is_empty() {
                return Ok(Bytes::new());
            }
            let restored = replacer::restore(&remaining, &map_for_flush);
            debug!("stream flush {} bytes → {} bytes", remaining.len(), restored.len());
            let delta = serde_json::json!({
                "choices": [{"delta": {"content": restored}, "index": 0}]
            });
            let output = format!("data: {}\n\n", serde_json::to_string(&delta).unwrap());
            Ok::<_, axum::Error>(Bytes::from(output))
        }));

    (status, headers, Body::from_stream(stream))
}

/// 处理单个 SSE chunk（可能包含多条消息）
fn process_sse_chunk(chunk_str: &str, text_buf: &Mutex<String>, map: &PlaceholderMap) -> String {
    // 按 \n\n 分割 SSE 消息
    let mut output = String::new();
    let messages: Vec<&str> = chunk_str.split("\n\n").collect();

    for (i, msg) in messages.iter().enumerate() {
        if msg.is_empty() {
            continue;
        }
        let msg_with_nl = if i < messages.len() - 1 {
            format!("{}\n\n", msg)
        } else {
            // 最后一个消息：检查原 chunk 是否以 \n\n 结尾
            if chunk_str.ends_with("\n\n") {
                format!("{}\n\n", msg)
            } else {
                msg.to_string()
            }
        };

        let processed = process_one_message(&msg_with_nl, text_buf, map);
        output.push_str(&processed);
    }

    output
}

/// 处理单条 SSE 消息
fn process_one_message(msg: &str, text_buf: &Mutex<String>, map: &PlaceholderMap) -> String {
    let trimmed = msg.trim();

    // [DONE] 信号 — 原样转发
    if trimmed == "data: [DONE]" {
        return msg.to_string();
    }

    // 提取 data: 后面的 JSON
    let json_str = match trimmed.strip_prefix("data: ") {
        Some(s) => s,
        None => return msg.to_string(),
    };

    // 解析 JSON
    let mut value: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return msg.to_string(),
    };

    // 尝试提取 content 或 tool_call function.arguments
    let text_field = extract_text_field(&value);

    let (field_path, text_to_add) = match text_field {
        Some((path, s)) if !s.is_empty() => (path, s.to_string()),
        _ => return msg.to_string(), // 无文本内容 → 原样转发
    };

    // 累积到文本缓冲区
    let mut buf = text_buf.lock().unwrap();
    buf.push_str(&text_to_add);

    // 在纯文本上找安全分割点
    let safe_len = find_safe_len(&buf, map);

    if safe_len == 0 {
        debug!("text buffer all unsafe, holding {} bytes", buf.len());
        return String::new();
    }

    let safe_text = buf[..safe_len].to_string();
    *buf = buf[safe_len..].to_string();

    let restored = replacer::restore(&safe_text, map);

    // 修改对应字段
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

/// 从 delta JSON 中提取文本内容，优先 content，其次 tool_call function.arguments
fn extract_text_field(value: &Value) -> Option<(String, &str)> {
    // 优先处理 content
    if let Some(s) = value
        .pointer("/choices/0/delta/content")
        .and_then(|v| v.as_str())
    {
        if !s.is_empty() {
            return Some(("/choices/0/delta/content".to_string(), s));
        }
    }

    // 其次处理 tool_calls[0].function.arguments
    if let Some(s) = value
        .pointer("/choices/0/delta/tool_calls/0/function/arguments")
        .and_then(|v| v.as_str())
    {
        if !s.is_empty() {
            return Some((
                "/choices/0/delta/tool_calls/0/function/arguments".to_string(),
                s,
            ));
        }
    }

    None
}

/// 在文本中寻找安全分割点：返回可安全还原并转发的字节位置。
///
/// 检查文本尾部是否匹配某个已知占位符的前缀。
/// 若匹配 → 保留该前缀等后续 chunk 补全，安全位置在前缀之前。
/// 若不匹配 → 全部安全。
fn find_safe_len(text: &str, map: &PlaceholderMap) -> usize {
    let mut keep_from = text.len();

    for placeholder in map.placeholders() {
        // 前缀长度从 1 到 len-1（不含完整占位符本身）
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
        // [[PHONE_CN@ 是占位符前缀，应保留
        let trailing = &text[safe..];
        assert!(trailing.starts_with("[[PHONE_CN@"));
    }

    #[test]
    fn test_find_safe_len_complete_placeholder() {
        let map = make_map();
        // 获取完整占位符
        let placeholder = map.placeholders().next().unwrap().clone();
        let text = format!("我的手机号 {}", placeholder);
        // 完整占位符在末尾 → 前缀检查不会匹配完整长度 → 全部安全
        assert_eq!(find_safe_len(&text, &map), text.len());
    }

    #[test]
    fn test_find_safe_len_complete_then_incomplete() {
        let map = make_map();
        let placeholder = map.placeholders().next().unwrap().clone();
        let text = format!("我的手机号 {} 另一个 [", placeholder);
        // 尾部 "[" 是占位符前缀 → 安全点在 "[" 之前
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
