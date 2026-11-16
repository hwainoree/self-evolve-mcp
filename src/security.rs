use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use url::Url;

pub const PUBLIC_ORIGIN: &str = "https://mcm.hn.yaaha.net";

pub fn valid_origin(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https") && url.origin().ascii_serialization() == value
}

pub fn allowed_origin(headers: &HeaderMap, accepted: &str) -> bool {
    let mut origins = headers.get_all("origin").iter();
    let Some(first) = origins.next() else {
        return true;
    };
    if origins.next().is_some() {
        return false;
    }
    let Ok(value) = first.to_str() else {
        return false;
    };
    if !valid_origin(value) {
        return false;
    }
    if value == accepted {
        return true;
    }
    let Ok(url) = Url::parse(value) else {
        return false;
    };
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

#[derive(Clone)]
pub struct RequestPolicy {
    pub accepted_origin: String,
    pub slots: std::sync::Arc<tokio::sync::Semaphore>,
    pub timeout: std::time::Duration,
}

impl RequestPolicy {
    pub fn new(
        accepted_origin: String,
        concurrent_requests: usize,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            accepted_origin,
            slots: std::sync::Arc::new(tokio::sync::Semaphore::new(concurrent_requests)),
            timeout,
        }
    }
}

pub async fn guard(
    axum::extract::State(policy): axum::extract::State<RequestPolicy>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !allowed_origin(request.headers(), &policy.accepted_origin) {
        return Err(StatusCode::FORBIDDEN);
    }
    let _permit = policy
        .slots
        .try_acquire()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    tokio::time::timeout(policy.timeout, next.run(request))
        .await
        .map_err(|_| StatusCode::REQUEST_TIMEOUT)
}
