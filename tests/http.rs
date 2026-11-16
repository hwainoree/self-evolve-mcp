use serde_json::{Value, json};

async fn spawn() -> (reqwest::Client, String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(listener, self_evolve_mcp::app()).await.unwrap();
    });
    (reqwest::Client::new(), url, handle)
}

#[tokio::test]
async fn health_is_ok() {
    let (client, url, handle) = spawn().await;
    let response = client.get(format!("{url}/healthz")).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "ok");
    handle.abort();
}

#[tokio::test]
async fn root_advertises_mcp_endpoint() {
    let (client, url, handle) = spawn().await;
    let response = client.get(&url).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body, json!({"name":"self-evolve-mcp","endpoint":"/mcp"}));
    handle.abort();
}

async fn rpc(client: &reqwest::Client, url: &str, body: Value) -> Value {
    let response = client
        .post(format!("{url}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-11-25")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response.headers().get("mcp-session-id").is_none(),
        "stateless transport must not allocate sessions"
    );
    response.json().await.unwrap()
}

#[tokio::test]
async fn mcp_initializes_lists_only_marco_and_returns_polo() {
    let (client, url, handle) = spawn().await;
    let init = rpc(&client, &url, json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"integration-test","version":"1"}}})).await;
    assert_eq!(init["result"]["serverInfo"]["name"], "self-evolve-mcp");
    assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
    let notification = client
        .post(format!("{url}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-11-25")
        .json(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
        .send()
        .await
        .unwrap();
    assert_eq!(notification.status(), 202);
    let list = rpc(
        &client,
        &url,
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await;
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "marco");
    assert_eq!(tools[0]["inputSchema"]["properties"], json!({}));
    let call = rpc(&client, &url, json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"marco","arguments":{}}})).await;
    assert_eq!(
        call["result"]["content"],
        json!([{"type":"text","text":"polo"}])
    );
    assert_ne!(call["result"]["isError"], true);
    handle.abort();
}

#[tokio::test]
async fn untrusted_and_malformed_origins_are_forbidden() {
    let (client, url, handle) = spawn().await;
    for origin in [
        "https://evil.example",
        "null",
        "https://mcm.hn.yaaha.net.evil.example",
        "https://mcm.hn.yaaha.net/path",
        "http://localhost.evil.example",
        "http://user@localhost:3000",
        "http://localhost:3000/",
        "http://localhost:3000 https://evil.example",
    ] {
        let response = client
            .post(format!("{url}/mcp"))
            .header("Origin", origin)
            .header("Accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 403, "origin: {origin}");
    }
    let response = client
        .get(&url)
        .header("Origin", "http://localhost:3000")
        .header("Origin", "https://evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 403, "multiple Origin headers");
    handle.abort();
}

#[tokio::test]
async fn oversized_body_is_rejected() {
    let (client, url, handle) = spawn().await;
    let response = client
        .post(format!("{url}/mcp"))
        .header("Accept", "application/json, text/event-stream")
        .header("Content-Type", "application/json")
        .body(" ".repeat(65537))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 413);
    handle.abort();
}

#[tokio::test]
async fn configured_origin_and_loopback_origins_are_accepted() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let config =
        self_evolve_mcp::config::Config::from_values(None, Some("https://example.org")).unwrap();
    let router =
        self_evolve_mcp::app_with_config(config, tokio_util::sync::CancellationToken::new());
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = reqwest::Client::new();
    for origin in [
        "https://example.org",
        "http://localhost:3000",
        "https://127.0.0.1:8443",
        "http://[::1]:3000",
    ] {
        let response = client
            .get(&url)
            .header("Origin", origin)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{origin}");
    }
    assert_eq!(
        client
            .get(&url)
            .header("Origin", "https://mcm.hn.yaaha.net")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    handle.abort();
}

#[tokio::test]
async fn unknown_tool_returns_sdk_error() {
    let (client, url, handle) = spawn().await;
    let response = rpc(&client, &url, json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"does-not-exist","arguments":{}}})).await;
    assert!(response["error"]["code"].is_number());
    assert!(response.get("result").is_none());
    handle.abort();
}
