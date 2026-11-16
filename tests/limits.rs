use axum::{Router, middleware, routing::get};
use self_evolve_mcp::security::{PUBLIC_ORIGIN, RequestPolicy, guard};
use std::time::Duration;

async fn spawn(policy: RequestPolicy) -> (String, tokio::task::JoinHandle<()>) {
    let router = Router::new()
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                "done"
            }),
        )
        .layer(middleware::from_fn_with_state(policy, guard));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/slow", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (url, handle)
}

#[tokio::test]
async fn saturated_requests_are_rejected_without_queueing() {
    let policy = RequestPolicy::new(PUBLIC_ORIGIN.into(), 1, Duration::from_secs(1));
    let slots = policy.slots.clone();
    let permit = slots.acquire().await.unwrap();
    let (url, handle) = spawn(policy).await;
    let client = reqwest::Client::new();
    assert_eq!(client.get(&url).send().await.unwrap().status(), 503);
    drop(permit);
    assert_eq!(client.get(&url).send().await.unwrap().status(), 200);
    handle.abort();
}

#[tokio::test]
async fn slow_requests_timeout() {
    let policy = RequestPolicy::new(PUBLIC_ORIGIN.into(), 1, Duration::from_millis(20));
    let (url, handle) = spawn(policy).await;
    let response = reqwest::Client::new().get(&url).send().await.unwrap();
    assert_eq!(response.status(), 408);
    handle.abort();
}
