use serde_json::json;

#[tokio::test]
async fn configured_public_host_accepts_real_mcp_calls_without_disabling_host_checks() {
    for origin in ["https://mcm.hn.yaaha.net", "https://example.org"] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let config = self_evolve_mcp::config::Config::from_values(None, Some(origin)).unwrap();
        let router =
            self_evolve_mcp::app_with_config(config, tokio_util::sync::CancellationToken::new());
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let client = reqwest::Client::new();
        let host = origin.strip_prefix("https://").unwrap();
        let result = client.post(format!("{url}/mcp"))
            .header("Host", host)
            .header("Accept", "application/json, text/event-stream")
            .header("MCP-Protocol-Version", "2025-11-25")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"marco","arguments":{}}}))
            .send().await.unwrap();
        assert_eq!(result.status(), 200, "configured public host: {host}");
        let body: serde_json::Value = result.json().await.unwrap();
        assert_eq!(body["result"]["content"][0]["text"], "polo");
        let forbidden = client
            .post(format!("{url}/mcp"))
            .header("Host", "evil.example")
            .header("Accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(forbidden.status(), 403);
        handle.abort();
    }
}
