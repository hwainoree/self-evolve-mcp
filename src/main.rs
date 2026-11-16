use self_evolve_mcp::{app_with_config, config::Config};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = std::env::var("BIND_ADDR").ok();
    let origin = std::env::var("ACCEPTED_ORIGIN").ok();
    let config = Config::from_values(bind.as_deref(), origin.as_deref())?;
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let cancellation = CancellationToken::new();
    let app = app_with_config(config, cancellation.clone());
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    eprintln!(
        "self-evolve-mcp listening on {} (MCP: /mcp)",
        listener.local_addr()?
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::select! {
                result = tokio::signal::ctrl_c() => {
                    if let Err(error) = result {
                        eprintln!("failed to listen for Ctrl-C: {error}");
                    }
                }
                _ = terminate.recv() => {}
            }
            cancellation.cancel();
        })
        .await?;
    Ok(())
}
