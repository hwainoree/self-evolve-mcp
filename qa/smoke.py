#!/usr/bin/env python3
"""Exercise a real remote MCP server; no credentials or mocked responses."""
import argparse
import json
import urllib.error
import urllib.request
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('base', nargs='?', default='https://mcm.hn.yaaha.net')
parser.add_argument('--output')
args = parser.parse_args()
base = args.base.rstrip('/')
checks = []
headers = {'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream'}

def rpc(method, params=None, ident=1):
    body = {'jsonrpc': '2.0', 'id': ident, 'method': method}
    if params is not None:
        body['params'] = params
    request = urllib.request.Request(base+'/mcp', data=json.dumps(body).encode(), headers=headers)
    with urllib.request.urlopen(request, timeout=15) as response:
        assert response.status == 200
        assert response.headers.get('Mcp-Session-Id') is None, 'expected stateless transport'
        raw = response.read().decode()
        if response.headers.get('Content-Type', '').startswith('text/event-stream'):
            raw = next(line[6:] for line in raw.splitlines() if line.startswith('data: '))
        return json.loads(raw)

with urllib.request.urlopen(base+'/healthz', timeout=15) as response:
    assert response.status == 200
checks.append('trusted HTTP(S) health')
with urllib.request.urlopen(base+'/', timeout=15) as response:
    assert json.load(response)['endpoint'] == '/mcp'
checks.append('base URL advertises MCP endpoint')
initialized = rpc('initialize', {'protocolVersion': '2025-11-25', 'capabilities': {}, 'clientInfo': {'name': 'smoke', 'version': '1.0'}})
assert initialized['result']['serverInfo']['name'] == 'self-evolve-mcp', initialized
checks.append('real MCP initialize')
headers['MCP-Protocol-Version'] = initialized['result']['protocolVersion']
request = urllib.request.Request(base+'/mcp', data=json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'}).encode(), headers=headers)
with urllib.request.urlopen(request, timeout=15) as response:
    assert response.status in (200, 202, 204)
checks.append('initialized notification accepted')
listed = rpc('tools/list')['result']['tools']
assert [tool['name'] for tool in listed] == ['marco'], listed
checks.append('exactly one discoverable tool: marco')
result = rpc('tools/call', {'name': 'marco', 'arguments': {}})['result']
assert result['content'] == [{'type': 'text', 'text': 'polo'}], result
assert not result.get('isError', False)
checks.append('real tools/call marco returns polo')
unknown = rpc('tools/call', {'name': 'missing', 'arguments': {}})
assert 'error' in unknown or unknown.get('result', {}).get('isError'), unknown
checks.append('unknown tool rejected')
request = urllib.request.Request(base+'/mcp', data=b'{}', headers={**headers, 'Origin': 'https://evil.example'})
try:
    urllib.request.urlopen(request, timeout=15)
    raise AssertionError('untrusted Origin was accepted')
except urllib.error.HTTPError as error:
    assert error.code == 403, error.code
checks.append('untrusted Origin rejected')
report = {'base': base, 'checks': checks, 'protocol': initialized['result']['protocolVersion'], 'marco': result}
if args.output:
    Path(args.output).write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps(report, indent=2))
