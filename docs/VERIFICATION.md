# Verification and measured performance

Release **v0.1.1**, official rmcp 3.10.0, Axum/Tokio, stateless JSON Streamable HTTP.

- **13 Rust tests** pass; formatting and Clippy (`-D warnings`) pass.
- Release binary startup, health, real MCP lifecycle and clean signal shutdown exercised.
- Independent initial source and public-Host fix reviews passed.
- Eight public HTTPS checks: health, base discovery, initialization, initialized notification, exactly one listed tool, marco call → polo, unknown-tool rejection, untrusted-Origin rejection.
- Protocol negotiated: `2025-11-25`; no allocated MCP session IDs.
- Actual GitHub release download and deployed init-container checksum checks passed.

## Measurements

Python stdlib keep-alive client, eight workers. These measurements include Python scheduling, serialization and connection overhead; not a maximum-capacity benchmark.

- Initial local ARM64 pod loopback (v0.1.0): 2,000 successful calls, approximately 3,961 calls/sec, median 1.74 ms, p95 3.81 ms.
- Deployed v0.1.1 over trusted HTTPS via Traefik: 200 successful calls, approximately 845 calls/sec, median 3.65 ms, p95 7.57 ms.
- Stripped release binary: **3,808,336 bytes** (v0.1.1).
- Initial local process after load: RSS **6,476 kB**, five threads. A point-in-time local observation, not a production memory guarantee.

No failures in these bounded runs. No sustained load test, adversarial connection test or fleet-wide capacity claim. Application bounds (64 handlers, 64 KiB request body, 10-second deadline) are not complete edge-level DoS protection.

## Reproduce

```sh
python3 qa/smoke.py https://mcm.hn.yaaha.net
python3 qa/bench.py https://mcm.hn.yaaha.net --requests 200 --concurrency 8
```

The Dockerfile is included. No local Docker daemon was accessible, so no local image-build claim is made. GitHub Actions runs formatting, Clippy, tests, binary smoke and Docker build; consult its actual status before claiming CI success. Current deployment runs the checksum-pinned released binary using pinned runtime images.
