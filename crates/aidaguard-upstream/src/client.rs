use reqwest::{Client, Method, Response};
use std::time::Duration;
use tracing::error;

use crate::types::{AuthType, ProviderConfig};

/// Unified HTTP client for any LLM provider.
///
/// Handles auth header construction based on `AuthType`
/// (Bearer token vs custom header like x-api-key).
#[derive(Clone)]
pub struct UpstreamClient {
    client: Client,
    provider: ProviderConfig,
    api_key: String,
    timeout: Duration,
}

impl UpstreamClient {
    pub fn new(provider: ProviderConfig, api_key: String, timeout_secs: u64) -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().use_rustls_tls().build()?,
            provider,
            api_key,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// Return the provider id this client is bound to.
    pub fn provider_id(&self) -> &str {
        &self.provider.id
    }

    /// Return the provider endpoint base URL.
    pub fn endpoint(&self) -> &str {
        &self.provider.endpoint
    }

    /// Build the auth header(s) for this provider.
    fn auth_headers(&self) -> Vec<(String, String)> {
        match &self.provider.auth {
            AuthType::BearerToken => {
                let value = if self.api_key.starts_with("Bearer ") {
                    self.api_key.clone()
                } else {
                    format!("Bearer {}", self.api_key)
                };
                vec![("authorization".to_string(), value)]
            }
            AuthType::ApiKeyHeader { header } => {
                vec![(header.clone(), self.api_key.clone())]
            }
        }
    }

    /// Forward a request to the upstream provider.
    pub async fn forward(
        &self,
        method: Method,
        url: &str,
        headers: reqwest::header::HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response, anyhow::Error> {
        let mut req = self
            .client
            .request(method, url)
            .headers(headers)
            .body(body)
            .timeout(self.timeout);

        for (key, value) in self.auth_headers() {
            req = req.header(key, value);
        }

        req.send().await.map_err(|e| {
            error!("Upstream request to {} failed: {}", self.provider.id, e);
            anyhow::anyhow!("upstream unreachable ({}): {}", self.provider.id, e)
        })
    }
}
