use axum::http::HeaderMap;
use reqwest::{Client, Method, Response};
use std::time::Duration;
use tracing::error;

/// 上游转发器，封装 HTTP 客户端与请求转发逻辑。
#[derive(Clone)]
pub struct Forwarder {
    client: Client,
    default_timeout: Duration,
}

impl Forwarder {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().use_rustls_tls().build()?,
            default_timeout: Duration::from_secs(300),
        })
    }

    /// 转发请求到上游，返回原始响应。
    ///
    /// 自动处理 `Bearer ` 前缀 — 若 api_key 不包含则自动添加。
    pub async fn forward(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
        api_key: &str,
    ) -> Result<Response, anyhow::Error> {
        let auth_value = if api_key.starts_with("Bearer ") {
            api_key.to_string()
        } else {
            format!("Bearer {}", api_key)
        };

        let resp = self
            .client
            .request(method, url)
            .headers(headers)
            .header("authorization", &auth_value)
            .body(body)
            .timeout(self.default_timeout)
            .send()
            .await
            .map_err(|e| {
                error!("Upstream request failed: {}", e);
                anyhow::anyhow!("upstream unreachable: {}", e)
            })?;

        Ok(resp)
    }

    /// 修改默认超时
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }
}
