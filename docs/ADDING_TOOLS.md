# Adding tools

Only `marco` is registered initially. New tools are ordinary Rust code; this service does not modify itself.

1. Write a failing HTTP integration test in `tests/http.rs`: verify the new name/schema through `tools/list` and its result/errors through `tools/call`.
2. Add `src/tools/<name>.rs`, following `marco.rs`. Each module implements methods on `McpServer` with the official `#[tool_router(router = <name>_router, vis = "pub(super)")]` and `#[tool(...)]` macros.
3. Declare `pub mod <name>;` in `src/tools/mod.rs`. Compose it into `McpServer::default()` using the SDK-supported router addition:
   `Self::marco_router() + Self::<name>_router()`.
4. For inputs, use the SDK's typed `Parameters<T>` wrapper and schema derivation, not manually decoded JSON. For state/dependencies, add explicit fields to `McpServer` and use shared bounded resources; stateless HTTP constructs a fresh handler per request.
5. Keep CPU/blocking I/O off async executor threads. Apply explicit timeouts, concurrency limits and cleanup to external calls; the outer 10-second handler timeout is a last resort, not a substitute for tool-specific cancellation.
6. Before tools that read private data, write files, execute commands or mutate infrastructure: add real authentication/authorization and scoped permissions. This initial public anonymous endpoint is safe only because `marco` is harmless.
7. Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --locked -j2 -- -D warnings`, `cargo test --locked -j2`, and the real MCP smoke test. Publish a new immutable version, update the deployment binary URL/checksum or container digest, and verify the public tool call.

The router-composition syntax is supported by rmcp 3.10.0's `ToolRouter` addition implementation. This guide adds no second tool to the running server.
