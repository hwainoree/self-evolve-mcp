# self-evolve-mcp

A small, extensible Rust MCP server using the official [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) SDK (pinned to 3.10.0) and Axum. It exposes exactly one tool: `marco`, taking no arguments and returning the text `polo`.

Despite the project name, there is **no self-modification, autonomous evolution, shell execution, or deployment functionality**. New tools are added by editing Rust source and rebuilding.

## Endpoints

| Path | Purpose |
| --- | --- |
| `/mcp` | Stateless MCP Streamable HTTP endpoint; use this exact path, not `/` |
| `/healthz` | Liveness/readiness response: HTTP 200, body `ok` |
| `/` | JSON service metadata advertising `/mcp` |

The intended public MCP URL is `https://mcm.hn.yaaha.net/mcp`. This documents a target URL, **not a claim that a deployment is currently live**. TLS termination, ingress, authentication, and publication are external responsibilities. See [DEPLOYMENT.md](DEPLOYMENT.md) for the current checksum-pinned ARM64 release-binary bootstrap using a Debian trixie runtime while registry access is pending; the Dockerfile provides an alternative source-build path.

## Build and run

Use Rust 1.100.0 (the toolchain tested by CI):

```sh
cargo build --release --locked -j2
BIND_ADDR=127.0.0.1:8080 ./target/release/self-evolve-mcp
curl --fail http://127.0.0.1:8080/healthz
```

| Environment variable | Default | Meaning |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:8080` | Numeric socket address to bind; IPv6 uses `[::1]:8080` syntax |
| `ACCEPTED_ORIGIN` | `https://mcm.hn.yaaha.net` | Canonical HTTP(S) origin, with no trailing slash, credentials, path, query, or fragment |

Invalid configuration or a bind failure exits nonzero. SIGTERM and Ctrl-C stop accepting requests, cancel MCP transport work, and allow Axum to shut down gracefully. Startup information is written to stderr.

## Try MCP

Requests must include `Content-Type: application/json` and `Accept: application/json, text/event-stream`. This server supports stateless JSON responses without allocating session IDs. For example:

```sh
curl --fail http://127.0.0.1:8080/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-11-25' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"curl","version":"1"}}}'

curl --fail http://127.0.0.1:8080/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-11-25' \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized"}'

curl --fail http://127.0.0.1:8080/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2025-11-25' \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"marco","arguments":{}}}'
```

The last response contains `"content":[{"type":"text","text":"polo"}]`. `tools/list` advertises only `marco`.

## Security and limits

- No authentication is implemented. `Origin` checks are not authentication: clients that omit `Origin` are permitted.
- The configured origin and canonical loopback origins (`localhost`, `127.0.0.1`, `[::1]`) are accepted. Malformed, duplicate, or untrusted origins receive HTTP 403.
- MCP request bodies are limited to 65,536 bytes (HTTP 413 above the limit).
- There are 64 simultaneous request-handler slots (HTTP 503 when exhausted) and a 10-second handler timeout (HTTP 408). These are handler limits, not a comprehensive slow-client or ingress defense.
- No permissive CORS headers are emitted. Origin validation does not enable cross-origin browser access.
- Use an external proxy for TLS, additional rate/body/time limits, and authentication if required.

## Container

```sh
docker build -t self-evolve-mcp:local .
docker run --rm -p 127.0.0.1:8080:8080 self-evolve-mcp:local
```

The multi-stage Dockerfile builds with `rust:1.100-trixie` and runs as UID/GID 10001 on `debian:trixie-slim`. The runtime contains no Rust toolchain. Matching Debian generations avoids a newer-glibc binary being copied into an older runtime. Local native binaries are host/architecture dependent; rebuild for the intended runtime rather than assuming Bookworm compatibility. Docker commands above are instructions, not proof that an image has been built or published.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -j2 -- -D warnings
cargo test --locked -j2
cargo build --release --locked -j2
python3 tests/smoke_binary.py
```

Integration tests exercise real HTTP, MCP initialization/tool listing/calls, origin validation, body/concurrency/timeout limits, environment configuration, and clean SIGTERM/SIGINT shutdown. The release smoke test launches the actual binary and checks health plus the MCP lifecycle. CI additionally builds the container; it does not deploy or publish it.

Add tools under `src/tools/`, register them with the SDK tool router, and add protocol-level integration tests before implementation. See [Adding tools](docs/ADDING_TOOLS.md) for the modular router-composition workflow.

## License

MIT; see [LICENSE](LICENSE).
