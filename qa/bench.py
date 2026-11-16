#!/usr/bin/env python3
"""Small reproducible keep-alive HTTP benchmark, not a performance guarantee."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import http.client
import json
import statistics
import time
from urllib.parse import urlparse

parser=argparse.ArgumentParser()
parser.add_argument('base', nargs='?', default='http://127.0.0.1:8080')
parser.add_argument('--requests', type=int, default=1000)
parser.add_argument('--concurrency', type=int, default=8)
args=parser.parse_args()
assert args.requests > 0 and args.concurrency > 0
url=urlparse(args.base)
body=json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'marco','arguments':{}}})

def worker(n):
    cls=http.client.HTTPSConnection if url.scheme=='https' else http.client.HTTPConnection
    connection=cls(url.hostname,url.port,timeout=15)
    times=[]
    try:
        for _ in range(n):
            start=time.perf_counter()
            connection.request('POST','/mcp',body,{'Content-Type':'application/json','Accept':'application/json, text/event-stream','MCP-Protocol-Version':'2025-11-25'})
            response=connection.getresponse()
            data=json.loads(response.read())
            assert response.status==200 and data['result']['content'][0]['text']=='polo',data
            times.append((time.perf_counter()-start)*1000)
    finally:
        connection.close()
    return times

counts=[args.requests//args.concurrency + (i < args.requests%args.concurrency) for i in range(args.concurrency)]
start=time.perf_counter()
with ThreadPoolExecutor(max_workers=args.concurrency) as pool:
    latencies=[latency for batch in pool.map(worker,counts) for latency in batch]
elapsed=time.perf_counter()-start
ordered=sorted(latencies)
print(json.dumps({'base':args.base,'successful_calls':len(latencies),'concurrency':args.concurrency,'elapsed_seconds':elapsed,'calls_per_second':len(latencies)/elapsed,'median_ms':statistics.median(latencies),'p95_ms':ordered[min(len(ordered)-1,int(len(ordered)*.95))],'note':'Includes Python client overhead; one environment, not a capacity guarantee.'},indent=2))
