"""Exercise the native release binary over HTTP; no third-party dependencies."""
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request

root = Path(__file__).resolve().parents[1]
binary = root / "target/release/self-evolve-mcp"
assert binary.is_file(), f"Build the release binary first: {binary}"
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    address = f"127.0.0.1:{sock.getsockname()[1]}"
base = f"http://{address}"
process = subprocess.Popen(
    [str(binary)],
    env={**os.environ, "BIND_ADDR": address, "ACCEPTED_ORIGIN": "https://example.org"},
)
try:
    deadline = time.monotonic() + 5
    while True:
        try:
            with urllib.request.urlopen(base + "/healthz", timeout=1) as response:
                assert response.status == 200
                assert response.read() == b"ok"
            break
        except urllib.error.URLError:
            assert process.poll() is None, "server exited before readiness"
            if time.monotonic() >= deadline:
                raise TimeoutError("server did not become ready")
            time.sleep(0.02)
    print("GET /healthz: 200 ok", flush=True)

    def rpc(body, expected_status=200):
        request = urllib.request.Request(
            base + "/mcp", data=json.dumps(body).encode(),
            headers={"Content-Type": "application/json",
                     "Accept": "application/json, text/event-stream",
                     "MCP-Protocol-Version": "2025-11-25",
                     "Origin": "https://example.org"},
        )
        with urllib.request.urlopen(request, timeout=5) as response:
            assert response.status == expected_status
            assert response.headers.get("mcp-session-id") is None
            payload = response.read()
            return json.loads(payload) if payload else None

    init = rpc({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-11-25", "capabilities": {},
        "clientInfo": {"name": "release-smoke", "version": "1"}}})
    assert init["result"]["serverInfo"]["name"] == "self-evolve-mcp"
    print("initialize:", json.dumps(init), flush=True)
    rpc({"jsonrpc": "2.0", "method": "notifications/initialized"}, 202)
    tools = rpc({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
    assert [tool["name"] for tool in tools["result"]["tools"]] == ["marco"]
    result = rpc({"jsonrpc": "2.0", "id": 3, "method": "tools/call",
                  "params": {"name": "marco", "arguments": {}}})
    assert result["result"]["content"] == [{"type": "text", "text": "polo"}]
    assert result["result"].get("isError") is not True
    print("tools/call marco:", json.dumps(result), flush=True)
    process.send_signal(signal.SIGTERM)
    assert process.wait(timeout=5) == 0
    print("SIGTERM: clean exit 0", flush=True)
    print(f"binary: {binary}; size: {binary.stat().st_size} bytes", flush=True)
finally:
    if process.poll() is None:
        process.kill()
        process.wait()
