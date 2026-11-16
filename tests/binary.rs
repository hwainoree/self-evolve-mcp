use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test]
async fn binary_reads_environment_serves_http_and_exits_cleanly_on_signals() {
    let binary = env!("CARGO_BIN_EXE_self-evolve-mcp");
    for signal in ["-TERM", "-INT"] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let bind = listener.local_addr().unwrap();
        drop(listener);
        let mut server = Server(
            Command::new(binary)
                .env("BIND_ADDR", bind.to_string())
                .env("ACCEPTED_ORIGIN", "https://example.org")
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let client = reqwest::Client::new();
        let url = format!("http://{bind}/healthz");
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Ok(response) = client
                    .get(&url)
                    .header("Origin", "https://example.org")
                    .send()
                    .await
                {
                    assert_eq!(response.status(), 200);
                    assert_eq!(response.text().await.unwrap(), "ok");
                    break;
                }
                assert!(
                    server.0.try_wait().unwrap().is_none(),
                    "server exited before readiness"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            client
                .get(&url)
                .header("Origin", "https://evil.example")
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert!(
            Command::new("kill")
                .args([signal, &server.0.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Some(status) = server.0.try_wait().unwrap() {
                    assert!(status.success(), "shutdown was not graceful: {status}");
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .unwrap();
    }
}
