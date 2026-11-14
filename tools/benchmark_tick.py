#!/usr/bin/env python3
"""Randomized paired native timings. Run after all builds/profilers have stopped."""
import argparse, datetime, hashlib, json, math, os, pathlib, platform, random, subprocess

CASES = ['hipster','hipster_unrecorded','hipster_updates','hipster_timeline','door6','flyer','bb','adder_idle','hipster_load']
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--variant',action='append',required=True,help='NAME=ABSOLUTE_BINARY')
p.add_argument('--mode',action='append',default=[],help='NAME=off|on (benchmark cache config)')
p.add_argument('--cases',nargs='+',default=CASES,choices=CASES)
p.add_argument('--rounds',type=int,default=21)
p.add_argument('--seed',type=int,default=20261114)
p.add_argument('--cpu',type=int,default=None)
p.add_argument('--target-ms',type=float,default=150)
p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args()
variants=dict(v.split('=',1) for v in a.variant); modes=dict(v.split('=',1) for v in a.mode)
assert a.rounds>0 and a.target_ms>0
allowed=sorted(os.sched_getaffinity(0));cpu=a.cpu if a.cpu is not None else allowed[0]
assert cpu in allowed
assert all(mode in ('off','on') for mode in modes.values()),'Shadow instrumentation is forbidden in timings'
for key in os.environ:
    assert not key.startswith('MC_TICK_TRACE_') and key!='MC_TICK_DEBUG_COMPARATOR','Diagnostics must be off'

def invoke(name,case,iterations):
    env=os.environ.copy();env['MC_TICK_POWER_CACHE']=modes.get(name,'off')
    command=['taskset','-c',str(cpu),variants[name],case,'1',str(iterations)]
    return json.loads(subprocess.check_output(command,env=env,text=True))

metadata={'kind':'metadata','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'host':platform.node(),'platform':platform.platform(),'cpu':cpu,'allowed_cpus':allowed,
    'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text().split('\n\n')[0],
    'seed':a.seed,'rounds':a.rounds,'target_ms':a.target_ms,'modes':modes,
    'variants':{name:{'path':path,'sha256':hashlib.file_digest(open(path,'rb'),'sha256').hexdigest()}
                for name,path in variants.items()},
    'method':'one warmup sample per subprocess; input+stepping+recording timed; restore/setup/drop excluded',
    'build':'Rust 1.98.0; --locked --release; opt-level=3; LTO=true; debug=1; generic x86-64; no PGO'}
a.output.parent.mkdir(parents=True,exist_ok=True)
rng=random.Random(a.seed)
with a.output.open('w') as f:
    def emit(row):f.write(json.dumps(row)+'\n');f.flush()
    emit(metadata)
    for case in a.cases:
        probe=invoke(next(iter(variants)),case,1)
        iterations=max(1,math.ceil(a.target_ms*1e6/probe['samples_cpu_ns'][0]))
        emit({'kind':'calibration','case':case,'probe':probe,'iterations':iterations})
        for round in range(a.rounds):
            order=list(variants);rng.shuffle(order)
            for name in order:
                result=invoke(name,case,iterations)
                emit({'kind':'sample','round':round,'variant':name,**result})
            print(f'{case}: round {round+1}/{a.rounds}, {iterations} iterations',flush=True)
