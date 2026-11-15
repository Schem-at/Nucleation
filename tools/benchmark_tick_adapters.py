#!/usr/bin/env python3
"""Pinned-CPU, interleaved adapter samples. Run after compilation/rendering stops.

Each process verifies the full suite and reset replay, performs one untimed warmup,
then one measured sample. All measured outputs are checked outside their timer.
This compares workloads on one binary, not different engines or vanilla speed.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess

ROOT = Path(__file__).resolve().parents[1]
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def save(path, value): path.write_text(json.dumps(value, indent=2)+'\n')

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--archive',type=Path,required=True)
    p.add_argument('--binary',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--rounds',type=int,default=21)
    p.add_argument('--cpu',type=int)
    p.add_argument('--target-ms',type=float,default=150)
    a=p.parse_args()
    if not 1<=a.rounds<=1000 or not 1<=a.target_ms<=10000: p.error('invalid sample bounds')
    allowed=sorted(os.sched_getaffinity(0))
    cpu=a.cpu if a.cpu is not None else allowed[0]
    if cpu not in allowed: p.error('CPU not permitted')
    os.sched_setaffinity(0,{cpu})
    a.output.mkdir(parents=True,exist_ok=False)
    (a.output/'runs').mkdir()
    archive=a.archive.resolve(); binary=a.binary.resolve()
    library=json.loads((archive/'library.json').read_text())
    by_sha={r['sha256']:r for r in library['entries']}
    cases=[]
    for profile in sorted((ROOT/'circuit-adapters').glob('*/adapter.json')):
        data=json.loads(profile.read_text()); source=(archive/by_sha[data['source_sha256']]['local_path']).resolve()
        if not source.is_relative_to(archive) or digest(source)!=data['source_sha256']: raise ValueError('source mismatch')
        cases.append({'id':data['id'],'profile':profile,'source':source,'suite':profile.with_name('suite.json')})
    def run(case,iterations,label):
        cmd=[str(binary),str(case['profile']),str(case['source']),str(case['suite']),'1',str(iterations)]
        result=subprocess.run(cmd,capture_output=True,text=True,timeout=600)
        (a.output/'runs'/f'{label}.stdout.json').write_text(result.stdout)
        (a.output/'runs'/f'{label}.stderr').write_text(result.stderr)
        data=json.loads(result.stdout)
        if result.returncode or data['status']!='functional_pass': raise RuntimeError(f'failed {label}')
        if digest(binary)!=binary_hash or data['runner_sha256']!=binary_hash: raise RuntimeError('binary changed during run')
        return data
    binary_hash=digest(binary)
    compiler=subprocess.check_output(['rustc','-Vv'],text=True)
    meta={'schema_version':1,'host':platform.node(),'platform':platform.platform(),'compiler':compiler,
          'cpu':cpu,'permitted_cpus':allowed,'cpuinfo':Path('/proc/cpuinfo').read_text(),
          'binary_sha256':binary_hash,'rounds':a.rounds,'seed':20261115,'target_ms':a.target_ms,
          'clock':'elapsed_monotonic','scope':'single-threaded adapter throughput; no rendering, no cross-engine speedup claim',
          'method':'one verified untimed warmup/sample process; reset and assertions outside timers; no dropped samples',
          'cases':{c['id']:{'profile_sha256':digest(c['profile']),'suite_sha256':digest(c['suite']),'source_sha256':digest(c['source'])} for c in cases}}
    save(a.output/'metadata.json',meta)
    for c in cases:
        probe=run(c,1,c['id']+'-calibration')
        ns=probe['benchmark']['samples_ns'][0]
        c['iterations']=max(1,min(1000,math.ceil(a.target_ms*1e6/max(ns,1))))
        c['vectors']=probe['vectors']; c['ticks']=probe['ticks_per_iteration']
    rng=random.Random(20261115); records=[]
    for round_no in range(a.rounds):
        order=list(cases); rng.shuffle(order)
        for c in order:
            data=run(c,c['iterations'],f'{round_no:02d}-{c["id"]}')
            records.append({'round':round_no,'case':c['id'],'iterations':c['iterations'],
                            'elapsed_ns':data['benchmark']['samples_ns'][0],
                            'vectors_per_iteration':c['vectors'],'ticks_per_iteration':c['ticks']})
            save(a.output/'records.json',records)
        print(f'round {round_no+1}/{a.rounds}',flush=True)
    summary={}
    for c in cases:
        values=[r['elapsed_ns']/r['iterations'] for r in records if r['case']==c['id']]
        median=statistics.median(values)
        summary[c['id']]={'samples':len(values),'iterations_per_sample':c['iterations'],
                          'median_ms_per_suite':median/1e6,'vectors_per_suite':c['vectors'],
                          'game_ticks_per_suite':c['ticks'],'operations_per_second':c['vectors']*1e9/median,
                          'simulated_ticks_per_second':c['ticks']*1e9/median,
                          'range_ms_per_suite':[min(values)/1e6,max(values)/1e6]}
    save(a.output/'summary.json',summary)
    print(json.dumps(summary,indent=2))

if __name__=='__main__': main()
