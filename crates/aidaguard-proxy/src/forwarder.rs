use axum::http::HeaderMap;
use reqwest::{Client, Method, Response};
use std::time::Duration;
use tracing::error;

/// Auth mode for upstream requests.
#[derive(Clone)]
enum AuthMode {
    /// Standard Bearer token (OpenAI-compatible).
    Bearer { api_key: String },
    /// Custom header-based auth (Anthropic x-api-key, Google x-goog-api-key, etc.).
    CustomHeader { header: String, api_key: String },
}

/// HTTP forwarder with auth-type awareness.
#[derive(Clone)]
pub struct Forwarder {
    client: Client,
    default_timeout: Duration,
    auth: Option<AuthMode>,
}

impl Forwarder {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().use_rustls_tls().build()?,
            default_timeout: Duration::from_secs(300),
            auth: None,
        })
    }

    /// Build a forwarder from upstream provider config.
    pub fn from_upstream(
        provider: &aidaguard_upstream::ProviderConfig,
        api_key: String,
        timeout_secs: u64,
    ) -> anyhow::Result<Self> {
        let auth = match &provider.auth {
            aidaguard_upstream::AuthType::BearerToken => {
                let value = if api_key.starts_with("Bearer ") {
                    api_key
                } else {
                    format!("Bearer {}", api_key)
                };
                Some(AuthMode::Bearer { api_key: value })
            }
            aidaguard_upstream::AuthType::ApiKeyHeader { header } => {
                Some(AuthMode::CustomHeader {
                    header: header.clone(),
                    api_key,
                })
            }
        };

        Ok(Self {
            client: Client::builder().use_rustls_tls().build()?,
            default_timeout: Duration::from_secs(timeout_secs),
            auth,
        })
    }

    /// Set Bearer auth on an existing forwarder (backward-compatible path).
    pub fn with_bearer_auth(mut self, api_key: String) -> Self {
        let value = if api_key.starts_with("Bearer ") {
            api_key
        } else {
            format!("Bearer {}", api_key)
        };
        self.auth = Some(AuthMode::Bearer { api_key: value });
        self
    }

    /// Forward a request to the upstream.
    ///
    /// When `auth` is configured via `from_upstream` or `with_bearer_auth`,
    /// the api_key parameter is ignored in favor of the stored auth.
    pub async fn forward(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
        api_key: &str,
    ) -> Result<Response, anyhow::Error> {
        let mut req = self
            .client
            .request(method, url)
            .headers(headers)
            .body(body)
            .timeout(self.default_timeout);

        match &self.auth {
            Some(AuthMode::Bearer { api_key: key }) => {
                req = req.header("authorization", key);
            }
            Some(AuthMode::CustomHeader { header, api_key: key }) => {
                req = req.header(header, key);
            }
            None => {
                // Fallback: legacy Bearer auth from api_key parameter
                let value = if api_key.starts_with("Bearer ") {
                    api_key.to_string()
                } else {
                    format!("Bearer {}", api_key)
                };
                req = req.header("authorization", &value);
            }
        }

        req.send().await.map_err(|e| {
            error!("Upstream request failed: {}", e);
            anyhow::anyhow!("upstream unreachable: {}", e)
        })
    }

    /// Modify the default timeout.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }
}
