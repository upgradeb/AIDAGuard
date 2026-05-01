/// Mock SSE 服务器 — 模拟千帆 API 流式返回，用于测试 Aidaguard 流式透传
///
/// 启动: cargo run --bin mock-sse [端口号，默认 19999]
use axum::{
    body::Body,
    http::StatusCode,
    response::Response,
    routing::{any, get},
    Router,
};
use futures::{stream, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

const CHUNKS: &[&str] = &[
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"你好"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"，我是"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"千帆"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"助手"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"，很高兴"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"为您服务！"}}]}"#,
    r#"{"id":"test-001","object":"chat.completion.chunk","choices":[{"index":0,"delta":{},"finish_reason":"stop"}}]}"#,
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(19999);

    let app = Router::new()
        .route("/", get(health))
        .route("/v2/coding", any(chat_completions))
        .route("/v2/coding/chat/completions", any(chat_completions))
        .route("/v2/coding/*path", any(chat_completions));

    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Mock SSE server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "Mock SSE server running\n"
}

async fn chat_completions(req: axum::extract::Request) -> Response {
    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    if let Ok(text) = std::str::from_utf8(&body_bytes) {
        info!("📨 Request: {}", text);
    }

    let sse_stream = stream::iter(CHUNKS)
        .then(|chunk| async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let data = format!("data: {}\n\n", chunk);
            Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(data))
        })
        .chain(stream::once(async {
            Ok::<_, std::convert::Infallible>(
                axum::body::Bytes::from("data: [DONE]\n\n"),
            )
        }));

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(Body::from_stream(sse_stream))
        .unwrap()
}
