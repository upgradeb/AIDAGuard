//! Protocol conversion between OpenAI Responses API and Chat Completions API.
//!
//! Codex CLI uses the Responses API (`/v1/responses`), but most LLM providers
//! only support Chat Completions (`/v1/chat/completions`). This module converts
//! between the two formats so Codex can work with any Chat Completions provider.

use serde_json::{json, Value};
use tracing::debug;

/// Convert a Responses API request body to Chat Completions format.
pub fn convert_request_responses_to_chat(body: &Value) -> Value {
    let mut chat = serde_json::Map::new();

    // model
    if let Some(model) = body.get("model") {
        chat.insert("model".into(), model.clone());
    }

    // instructions → system message prefix
    let mut messages = Vec::new();
    if let Some(instructions) = body.get("instructions").and_then(|v| v.as_str()) {
        if !instructions.is_empty() {
            messages.push(json!({"role": "system", "content": instructions}));
        }
    }

    // input → messages
    if let Some(input) = body.get("input") {
        if let Some(arr) = input.as_array() {
            for item in arr {
                messages.push(convert_input_item_to_message(item));
            }
        } else if let Some(text) = input.as_str() {
            messages.push(json!({"role": "user", "content": text}));
        }
    }

    chat.insert("messages".into(), Value::Array(messages));

    // tools: Responses API uses same format but wrapped differently
    if let Some(tools) = body.get("tools") {
        let chat_tools = convert_tools_responses_to_chat(tools);
        chat.insert("tools".into(), chat_tools);
    }

    // tool_choice
    if let Some(tc) = body.get("tool_choice") {
        chat.insert("tool_choice".into(), tc.clone());
    }

    // parallel_tool_calls
    if let Some(ptc) = body.get("parallel_tool_calls") {
        chat.insert("parallel_tool_calls".into(), ptc.clone());
    }

    // max_output_tokens → max_completion_tokens
    if let Some(max) = body.get("max_output_tokens") {
        chat.insert("max_completion_tokens".into(), max.clone());
    }

    // temperature
    if let Some(temp) = body.get("temperature") {
        chat.insert("temperature".into(), temp.clone());
    }

    // top_p
    if let Some(top_p) = body.get("top_p") {
        chat.insert("top_p".into(), top_p.clone());
    }

    // stream
    if let Some(stream) = body.get("stream") {
        chat.insert("stream".into(), stream.clone());
    }

    // reasoning.effort → reasoning_effort
    if let Some(effort) = body.get("reasoning").and_then(|r| r.get("effort")) {
        chat.insert("reasoning_effort".into(), effort.clone());
    }

    // text.format → response_format
    if let Some(format) = body.get("text").and_then(|t| t.get("format")) {
        chat.insert("response_format".into(), format.clone());
    }

    // store
    if let Some(store) = body.get("store") {
        chat.insert("store".into(), store.clone());
    }

    // metadata / user
    if let Some(user) = body.get("user") {
        chat.insert("user".into(), user.clone());
    }

    Value::Object(chat)
}

/// Convert a single Responses API input item to a Chat Completions message.
fn convert_input_item_to_message(item: &Value) -> Value {
    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("message");

    match item_type {
        "message" => {
            let role = item.get("role").and_then(|v| v.as_str()).unwrap_or("user");
            let content = item.get("content");

            match content {
                Some(c) if c.is_string() => {
                    json!({"role": role, "content": c})
                }
                Some(c) if c.is_array() => {
                    // Extract text from content parts
                    let text = extract_text_from_content_parts(c);
                    json!({"role": role, "content": text})
                }
                _ => {
                    json!({"role": role, "content": ""})
                }
            }
        }
        "function_call" => {
            let call_id = item.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = item.get("arguments").and_then(|v| v.as_str()).unwrap_or("");
            json!({
                "role": "assistant",
                "tool_calls": [{
                    "id": call_id,
                    "type": "function",
                    "function": {"name": name, "arguments": arguments}
                }]
            })
        }
        "function_call_output" => {
            let call_id = item.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
            let output = item.get("output");
            let content = match output {
                Some(o) if o.is_string() => o.as_str().unwrap_or("").to_string(),
                Some(o) => serde_json::to_string(o).unwrap_or_default(),
                None => String::new(),
            };
            json!({"role": "tool", "tool_call_id": call_id, "content": content})
        }
        // Unknown types: try to extract as a user message
        _ => {
            debug!("Unknown input item type: {}, treating as user message", item_type);
            json!({"role": "user", "content": serde_json::to_string(item).unwrap_or_default()})
        }
    }
}

/// Extract plain text from Responses API content parts array.
fn extract_text_from_content_parts(parts: &Value) -> String {
    let mut text = String::new();
    if let Some(arr) = parts.as_array() {
        for part in arr {
            if let Some(t) = part.get("text").and_then(|v| v.as_str()) {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(t);
            }
        }
    }
    text
}

/// Convert Responses API tools to Chat Completions tools format.
fn convert_tools_responses_to_chat(tools: &Value) -> Value {
    let Some(arr) = tools.as_array() else {
        return tools.clone();
    };

    let converted: Vec<Value> = arr.iter().map(|tool| {
        let tool_type = tool.get("type").and_then(|v| v.as_str()).unwrap_or("function");

        match tool_type {
            "function" => {
                // Responses API function tools are similar but top-level
                // Chat Completions wraps in {"type": "function", "function": {...}}
                if tool.get("function").is_some() {
                    tool.clone()
                } else {
                    let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let description = tool.get("description").and_then(|v| v.as_str()).unwrap_or("");
                    let parameters = tool.get("parameters").cloned().unwrap_or(json!({}));
                    let strict = tool.get("strict").cloned().unwrap_or(json!(false));
                    json!({
                        "type": "function",
                        "function": {
                            "name": name,
                            "description": description,
                            "parameters": parameters,
                            "strict": strict,
                        }
                    })
                }
            }
            _ => tool.clone(),
        }
    }).collect();

    Value::Array(converted)
}

/// Convert a Chat Completions response body to Responses API format.
pub fn convert_response_chat_to_responses(body: &Value) -> Value {
    let mut resp = serde_json::Map::new();

    // id
    let chat_id = body.get("id").and_then(|v| v.as_str()).unwrap_or("chatcmpl-unknown");
    resp.insert("id".into(), json!(format!("resp_{}", chat_id.replace("chatcmpl-", ""))));
    resp.insert("object".into(), json!("response"));

    // model
    if let Some(model) = body.get("model") {
        resp.insert("model".into(), model.clone());
    }

    // created_at
    if let Some(created) = body.get("created").and_then(|v| v.as_i64()) {
        resp.insert("created_at".into(), json!(created));
    }

    // status
    resp.insert("status".into(), json!("completed"));

    // output: convert choices → output items
    let mut output = Vec::new();
    if let Some(choices) = body.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            if let Some(msg) = choice.get("message") {
                // Text content
                if let Some(content) = msg.get("content").and_then(|v| v.as_str()) {
                    if !content.is_empty() {
                        output.push(json!({
                            "type": "message",
                            "id": format!("msg_{}", uuid_short()),
                            "role": "assistant",
                            "status": "completed",
                            "content": [{"type": "output_text", "text": content}]
                        }));
                    }
                }

                // Tool calls
                if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                    for tc in tool_calls {
                        let call_id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = tc.get("function").and_then(|f| f.get("name")).and_then(|v| v.as_str()).unwrap_or("");
                        let arguments = tc.get("function").and_then(|f| f.get("arguments")).and_then(|v| v.as_str()).unwrap_or("");
                        output.push(json!({
                            "type": "function_call",
                            "id": format!("fc_{}", uuid_short()),
                            "call_id": call_id,
                            "name": name,
                            "arguments": arguments,
                            "status": "completed"
                        }));
                    }
                }
            }
        }
    }
    resp.insert("output".into(), Value::Array(output));

    // usage
    if let Some(usage) = body.get("usage") {
        resp.insert("usage".into(), convert_usage_chat_to_responses(usage));
    }

    resp.into()
}

/// Convert Chat Completions usage to Responses API usage format.
fn convert_usage_chat_to_responses(usage: &Value) -> Value {
    json!({
        "input_tokens": usage.get("prompt_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
        "output_tokens": usage.get("completion_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
        "total_tokens": usage.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
        "input_tokens_details": {
            "cached_tokens": usage.get("prompt_tokens_details")
                .and_then(|d| d.get("cached_tokens")).and_then(|v| v.as_i64()).unwrap_or(0)
        },
        "output_tokens_details": {
            "reasoning_tokens": usage.get("completion_tokens_details")
                .and_then(|d| d.get("reasoning_tokens")).and_then(|v| v.as_i64()).unwrap_or(0)
        }
    })
}

/// Convert a Chat Completions SSE event to Responses API SSE events.
///
/// Returns one or more SSE event strings (each prefixed with "data: " and suffixed with "\n\n").
/// Maintains state via the `state` parameter for multi-event sequences.
pub fn convert_stream_event_chat_to_responses(
    event: &Value,
    state: &mut StreamConvertState,
) -> Vec<String> {
    let mut events = Vec::new();

    // First chunk with role=assistant: emit lifecycle events
    if !state.started {
        if event.get("choices").is_some() {
            state.started = true;
            let resp_id = format!("resp_{}", uuid_short());
            state.response_id = resp_id.clone();
            state.item_id = format!("msg_{}", uuid_short());

            events.push(sse_event("response.created", json!({
                "response": {"id": resp_id, "object": "response", "status": "in_progress", "model": event.get("model").cloned().unwrap_or(json!(""))}
            })));

            events.push(sse_event("response.output_item.added", json!({
                "output_index": 0,
                "item": {"id": state.item_id, "type": "message", "role": "assistant", "status": "in_progress", "content": []}
            })));

            events.push(sse_event("response.content_part.added", json!({
                "output_index": 0,
                "content_index": 0,
                "item_id": state.item_id,
                "part": {"type": "output_text", "text": ""}
            })));
        }
    }

    // Process content delta
    if let Some(content) = event.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
        if !content.is_empty() {
            events.push(sse_event("response.output_text.delta", json!({
                "output_index": 0,
                "content_index": 0,
                "item_id": state.item_id,
                "delta": content
            })));
            state.text_buffer.push_str(content);
        }
    }

    // Process tool calls delta
    if let Some(tool_calls) = event.pointer("/choices/0/delta/tool_calls").and_then(|v| v.as_array()) {
        for tc in tool_calls {
            let tc_index = tc.get("index").and_then(|v| v.as_i64()).unwrap_or(0) as usize;

            // New tool call: emit output_item.added
            if tc.get("id").is_some() || tc.get("function").and_then(|f| f.get("name")).is_some() {
                let call_id_fallback = format!("call_{}", uuid_short());
                let call_id = tc.get("id").and_then(|v| v.as_str()).unwrap_or(&call_id_fallback);
                let name = tc.get("function").and_then(|f| f.get("name")).and_then(|v| v.as_str()).unwrap_or("");
                let fc_item_id = format!("fc_{}", uuid_short());

                state.tool_call_ids.push((tc_index, call_id.to_string(), fc_item_id.clone(), name.to_string()));

                events.push(sse_event("response.output_item.added", json!({
                    "output_index": 1 + tc_index,
                    "item": {
                        "type": "function_call",
                        "id": fc_item_id,
                        "call_id": call_id,
                        "name": name,
                        "arguments": "",
                        "status": "in_progress"
                    }
                })));
            }

            // Arguments delta
            if let Some(args) = tc.get("function").and_then(|f| f.get("arguments")).and_then(|v| v.as_str()) {
                let item_id = state.tool_call_ids.iter()
                    .find(|(idx, _, _, _)| *idx == tc_index)
                    .map(|(_, _, id, _)| id.as_str())
                    .unwrap_or("");

                events.push(sse_event("response.function_call_arguments.delta", json!({
                    "output_index": 1 + tc_index,
                    "item_id": item_id,
                    "delta": args
                })));
            }
        }
    }

    // Process finish_reason
    if let Some(finish_reason) = event.pointer("/choices/0/finish_reason").and_then(|v| v.as_str()) {
        if finish_reason != "null" {
            // Close text
            events.push(sse_event("response.output_text.done", json!({
                "output_index": 0,
                "content_index": 0,
                "item_id": state.item_id,
                "text": state.text_buffer
            })));

            // Close message item
            events.push(sse_event("response.output_item.done", json!({
                "output_index": 0,
                "item": {
                    "id": state.item_id,
                    "type": "message",
                    "role": "assistant",
                    "status": "completed",
                    "content": [{"type": "output_text", "text": state.text_buffer}]
                }
            })));

            // Close function calls
            for (idx, call_id, fc_id, name) in &state.tool_call_ids {
                events.push(sse_event("response.function_call_arguments.done", json!({
                    "output_index": 1 + idx,
                    "item_id": fc_id,
                    "arguments": "",
                    "name": name,
                    "call_id": call_id
                })));

                events.push(sse_event("response.output_item.done", json!({
                    "output_index": 1 + idx,
                    "item": {
                        "type": "function_call",
                        "id": fc_id,
                        "call_id": call_id,
                        "name": name,
                        "arguments": "",
                        "status": "completed"
                    }
                })));
            }

            // Response completed
            events.push(sse_event("response.completed", json!({
                "response": {
                    "id": state.response_id,
                    "object": "response",
                    "status": "completed",
                    "model": event.get("model").cloned().unwrap_or(json!("")),
                    "output": [],
                    "usage": {
                        "input_tokens": 0,
                        "output_tokens": 0,
                        "total_tokens": 0
                    }
                }
            })));
        }
    }

    events
}

/// State for stream event conversion (maintained across chunks).
#[derive(Default)]
pub struct StreamConvertState {
    started: bool,
    response_id: String,
    item_id: String,
    text_buffer: String,
    tool_call_ids: Vec<(usize, String, String, String)>, // (index, call_id, fc_item_id, name)
}

/// Rewrite the target URL from /v1/responses to /v1/chat/completions.
pub fn rewrite_url_responses_to_chat(url: &str) -> String {
    url.replace("/v1/responses", "/v1/chat/completions")
        .replace("/responses", "/chat/completions")
}

fn sse_event(event_type: &str, data: Value) -> String {
    let mut obj = data;
    if let Some(map) = obj.as_object_mut() {
        map.insert("type".into(), json!(event_type));
    }
    format!("data: {}\n\n", serde_json::to_string(&obj).unwrap_or_default())
}

fn uuid_short() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{:x}", ts % 0xFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_request() {
        let req = json!({
            "model": "gpt-4o",
            "input": [
                {"type": "message", "role": "user", "content": "Hello"}
            ],
            "stream": true
        });
        let chat = convert_request_responses_to_chat(&req);
        assert_eq!(chat["model"], "gpt-4o");
        assert_eq!(chat["stream"], true);
        let messages = chat["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
    }

    #[test]
    fn test_convert_request_with_instructions() {
        let req = json!({
            "model": "gpt-4o",
            "instructions": "You are helpful",
            "input": [
                {"type": "message", "role": "user", "content": "Hello"}
            ]
        });
        let chat = convert_request_responses_to_chat(&req);
        let messages = chat["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn test_convert_request_function_call_output() {
        let req = json!({
            "model": "gpt-4o",
            "input": [
                {"type": "function_call_output", "call_id": "call_123", "output": "result"}
            ]
        });
        let chat = convert_request_responses_to_chat(&req);
        let messages = chat["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "tool");
        assert_eq!(messages[0]["tool_call_id"], "call_123");
    }

    #[test]
    fn test_convert_response() {
        let chat_resp = json!({
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "model": "gpt-4o",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "Hello!"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        });
        let resp = convert_response_chat_to_responses(&chat_resp);
        assert_eq!(resp["object"], "response");
        assert_eq!(resp["status"], "completed");
        let output = resp["output"].as_array().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0]["type"], "message");
    }

    #[test]
    fn test_rewrite_url() {
        assert_eq!(
            rewrite_url_responses_to_chat("https://api.deepseek.com/v1/responses"),
            "https://api.deepseek.com/v1/chat/completions"
        );
    }
}
