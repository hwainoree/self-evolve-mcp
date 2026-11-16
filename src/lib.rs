pub mod config;
pub mod security;
pub mod tools;

use axum::{Json, Router, routing::get};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::json;

pub fn app() -> Router {
    app_with_config(
        config::Config::from_values(None, None).expect("valid defaults"),
        tokio_util::sync::CancellationToken::new(),
    )
}

pub fn app_with_config(config: config::Config, ct: tokio_util::sync::CancellationToken) -> Router {
    let public_url = url::Url::parse(&config.accepted_origin).expect("validated origin");
    let public_host = public_url.host_str().expect("origin has host").to_owned();
    let service: StreamableHttpService<tools::McpServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(tools::McpServer::default()),
            Default::default(),
            StreamableHttpServerConfig::default()
                .with_allowed_hosts([
                    public_host,
                    "localhost".into(),
                    "127.0.0.1".into(),
                    "::1".into(),
                ])
                .with_legacy_session_mode(false)
                .with_json_response(true)
                .with_max_request_body_bytes(65536)
                .with_sse_keep_alive(None)
                .with_cancellation_token(ct),
        );
    Router::new()
        .route(
            "/",
            get(|| async { Json(json!({"name":"self-evolve-mcp", "endpoint":"/mcp"})) }),
        )
        .route("/healthz", get(|| async { "ok" }))
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn_with_state(
            security::RequestPolicy::new(
                config.accepted_origin,
                64,
                std::time::Duration::from_secs(10),
            ),
            security::guard,
        ))
}
