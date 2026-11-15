#!/usr/bin/env python3
"""Resolve source-pinned adapter builds and run native checks/benchmarks. Never compiles."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--archive', type=Path, required=True)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--adapter', action='append')
    p.add_argument('--samples', type=int, default=0)
    p.add_argument('--iterations', type=int, default=1)
    p.add_argument('--timeout', type=int, default=600)
    a = p.parse_args()
    if not 0 <= a.samples <= 1000 or not 1 <= a.iterations <= 10000 or a.timeout <= 0:
        p.error('invalid sample, iteration or timeout bounds')
    archive = a.archive.resolve()
    library = json.loads((archive/'library.json').read_text())
    by_hash = {r['sha256']:r for r in library['entries'] if r['kind']=='schematic'}
    profiles = sorted((ROOT/'circuit-adapters').glob('*/adapter.json'))
    if a.adapter:
        profiles = [x for x in profiles if x.parent.name in a.adapter]
        if {x.parent.name for x in profiles} != set(a.adapter): p.error('unknown adapter')
    a.output.mkdir(parents=True,exist_ok=True)
    failures = 0
    for path in profiles:
        profile = json.loads(path.read_text())
        source = (archive/by_hash[profile['source_sha256']]['local_path']).resolve()
        if not source.is_relative_to(archive): raise ValueError('source escapes archive')
        if hashlib.sha256(source.read_bytes()).hexdigest()!=profile['source_sha256']:
            raise ValueError(f'changed original: {source}')
        out = a.output/f'{profile["id"]}.json'
        if out.exists(): raise FileExistsError(out)
        cmd = [str(a.binary.resolve()),str(path),str(source),str(path.with_name('suite.json'))]
        if a.samples: cmd += [str(a.samples),str(a.iterations)]
        try:
            result = subprocess.run(cmd,capture_output=True,text=True,timeout=a.timeout)
            data = json.loads(result.stdout)
            passed = result.returncode==0 and data.get('status')=='functional_pass'
            data['process_returncode']=result.returncode
        except subprocess.TimeoutExpired:
            data={'status':'timeout','timeout_seconds':a.timeout}; passed=False
            result=None
        out.write_text(json.dumps(data,indent=2)+'\n')
        if result: out.with_suffix('.stderr').write_text(result.stderr)
        print(f'{profile["id"]}: {data["status"]}',flush=True)
        failures += not passed
    return int(failures>0)

if __name__=='__main__': sys.exit(main())
