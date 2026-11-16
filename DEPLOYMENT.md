# Deployment

Target base URL: **https://mcm.hn.yaaha.net**
MCP client URL: **https://mcm.hn.yaaha.net/mcp** (Streamable HTTP).

Canonical source: `/workspace/hwainoree/self-evolve-mcp`.

## Current deployment path

Until a push-capable registry hostname/credentials and image builder are available, the manifest runs a **checksum-pinned released Rust binary**, not mounted source or a compiler:

1. Build/review/test the release binary and upload it as `self-evolve-mcp-linux-aarch64` in the public GitHub release `v0.1.0`.
2. `k8s/deployment.yaml` pins an ARM64 curl image and downloads the exact release asset via HTTPS into an emptyDir. A recorded SHA-256 is checked before execute permission is added.
3. A digest-pinned Debian trixie ARM64 runtime runs only that binary as UID 10001. No compiler, cargo install, source ConfigMaps or Kubernetes token in the workload.
4. Service `self-evolve-mcp` exposes port 80 to 8080. Traefik routes the HTTPS hostname using the existing TLS resolver.

The released Linux binary is ARM64 and built against glibc 2.41. These manifests are therefore ARM64-specific; do not deploy unchanged on x86 nodes. The Dockerfile is an alternative reproducible source-build path and can build for a target platform through a suitable image builder.

```sh
kubectl --kubeconfig=/opt/data/kubernetes/hwainoree.kubeconfig apply --dry-run=server -f k8s/deployment.yaml
kubectl --kubeconfig=/opt/data/kubernetes/hwainoree.kubeconfig apply -f k8s/deployment.yaml
kubectl --kubeconfig=/opt/data/kubernetes/hwainoree.kubeconfig -n hwainoree rollout status deployment/self-evolve-mcp
python3 qa/smoke.py https://mcm.hn.yaaha.net
```

Deployment edits are restricted to namespace `hwainoree`. Runtime filesystem is read-only; init can write only the shared binary directory. A new pod requires GitHub release availability during startup; already-running pods do not. Replica state is absent, so replicas can independently handle these stateless requests.

## Moving to your registry

Build the included multi-stage `Dockerfile` with a daemon/BuildKit or CI builder, push a versioned image, and pin its digest in `containers[].image`. Remove `fetch-binary`, its env vars and the runtime volume, and set the command to the image's installed binary (or use its entrypoint). Configure a namespace-scoped imagePullSecret only if required; do not commit credentials. Merely having a registry does not provide an image-building daemon.

## Security and scaling boundaries

Exactly one public harmless tool exists: `marco` returns `polo`. No command execution, self-editing, filesystem or network tools are implemented despite the project name. Before adding privileged tools, implement authorization, explicit permissions and credential scoping; Origin validation is not authentication. Browser CORS is intentionally not broadly enabled. MCP non-browser clients can connect without Origin.

Request body cap 64 KiB; up to 64 simultaneous handlers; 10-second handler deadline. These application limits are not a complete denial-of-service defense: use edge connection/rate limits and authenticate where appropriate. Blocking/CPU-heavy future tools must not run on Tokio workers; use bounded spawn_blocking/work queues. Streaming response bodies need their own lifetimes/backpressure if introduced later. This server deliberately returns JSON and allocates no persistent MCP sessions.
