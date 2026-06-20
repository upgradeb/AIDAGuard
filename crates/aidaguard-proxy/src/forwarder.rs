use axum::http::HeaderMap;
use reqwest::{Client, Method, Response};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::error;

/// Auth mode for upstream requests.
#[derive(Clone)]
enum AuthMode {
    Bearer { api_key: String },
    CustomHeader { header: String, api_key: String },
}

/// Simple per-request rate limiter.
/// Ensures at least `interval` time passes between consecutive requests.
#[derive(Clone)]
struct RateLimiter {
    state: Option<Arc<Mutex<RateLimitState>>>,
}

struct RateLimitState {
    last_request: Instant,
    interval: Duration,
}

impl RateLimiter {
    fn new(qps: u32) -> Self {
        let state = if qps > 0 {
            Some(Arc::new(Mutex::new(RateLimitState {
                last_request: Instant::now(),
                interval: Duration::from_secs_f64(1.0 / qps as f64),
            })))
        } else {
            None
        };
        Self { state }
    }

    /// Wait until the minimum interval has elapsed since the last request.
    async fn acquire(&self) {
        let Some(ref state) = self.state else { return };

        let mut s = state.lock().await;
        let elapsed = s.last_request.elapsed();
        if elapsed >= s.interval {
            s.last_request = Instant::now();
            return;
        }
        let remaining = s.interval - elapsed;
        s.last_request = Instant::now();
        drop(s);

        tokio::time::sleep(remaining).await;
    }
}

/// HTTP forwarder with auth-type awareness and optional rate limiting.
#[derive(Clone)]
pub struct Forwarder {
    client: Client,
    default_timeout: Duration,
    auth: Option<AuthMode>,
    rate_limiter: RateLimiter,
}

impl Forwarder {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().use_rustls_tls().build()?,
            default_timeout: Duration::from_secs(300),
            auth: None,
            rate_limiter: RateLimiter::new(0),
        })
    }

    /// Build a forwarder from upstream provider config.
    pub fn from_upstream(
        provider: &aidaguard_upstream::ProviderConfig,
        api_key: String,
        timeout_secs: u64,
        rate_limit_qps: u32,
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
            rate_limiter: RateLimiter::new(rate_limit_qps),
        })
    }

    /// Set Bearer auth on an existing forwarder.
    pub fn with_bearer_auth(mut self, api_key: String) -> Self {
        let value = if api_key.starts_with("Bearer ") {
            api_key
        } else {
            format!("Bearer {}", api_key)
        };
        self.auth = Some(AuthMode::Bearer { api_key: value });
        self
    }

    /// Set rate limiting on an existing forwarder.
    pub fn with_rate_limit(mut self, qps: u32) -> Self {
        self.rate_limiter = RateLimiter::new(qps);
        self
    }

    /// Forward a request to the upstream.
    /// If rate limiting is configured, waits for the minimum interval first.
    pub async fn forward(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
        api_key: &str,
    ) -> Result<Response, anyhow::Error> {
        // Enforce rate limit before sending
        self.rate_limiter.acquire().await;

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
