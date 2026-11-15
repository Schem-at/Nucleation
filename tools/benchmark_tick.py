#!/usr/bin/env python3
"""Randomized paired native timings. Run after all builds/profilers have stopped."""
import argparse, datetime, hashlib, json, math, os, pathlib, platform, random, subprocess

CASES = ['hipster','hipster_unrecorded','hipster_updates','hipster_timeline','door6','flyer','bb','adder_idle','hipster_load']
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--variant',action='append',required=True,help='NAME=ABSOLUTE_BINARY')
p.add_argument('--mode',action='append',default=[],help='NAME=off|on (benchmark cache config)')
p.add_argument('--plan-mode',action='append',default=[],help='NAME=off|on (wire input plans)')
p.add_argument('--cases',nargs='+',default=CASES,metavar='CASE')
p.add_argument('--rounds',type=int,default=21)
p.add_argument('--seed',type=int,default=20261114)
p.add_argument('--cpu',type=int,default=None)
p.add_argument('--target-ms',type=float,default=150)
p.add_argument('--output',type=pathlib.Path,required=True)
p.add_argument('--build-info',type=pathlib.Path,help='Producer metadata JSON; compiler/flags cannot be inferred from binaries')
a=p.parse_args()
variants=dict(v.split('=',1) for v in a.variant); modes=dict(v.split('=',1) for v in a.mode)
plan_modes=dict(v.split('=',1) for v in a.plan_mode)
assert a.rounds>0 and a.target_ms>0
allowed=sorted(os.sched_getaffinity(0));cpu=a.cpu if a.cpu is not None else allowed[0]
assert cpu in allowed
assert all(mode in ('off','on') for mode in modes.values()),'Shadow instrumentation is forbidden in timings'
assert all(mode in ('off','on') for mode in plan_modes.values()),'Shadow instrumentation is forbidden in timings'
assert os.environ.get('MC_TICK_WIRE_PLAN', 'on') != 'shadow', 'Shadow recomputation is forbidden in timings'
assert os.environ.get('MC_TICK_PISTON_PLAN', 'on') != 'shadow', 'Shadow recomputation is forbidden in timings'
assert os.environ.get('MC_TICK_TRANSITION_SIGNATURES', 'on') != 'shadow', 'Shadow recomputation is forbidden in timings'
for key in os.environ:
    assert not key.startswith('MC_TICK_TRACE_') and key!='MC_TICK_DEBUG_COMPARATOR','Diagnostics must be off'

def invoke(name,case,iterations):
    env=os.environ.copy();env['MC_TICK_POWER_CACHE']=modes.get(name,'off')
    env['MC_TICK_WIRE_PLAN']=plan_modes.get(name,env.get('MC_TICK_WIRE_PLAN','on'))
    command=['taskset','-c',str(cpu),variants[name],case,'1',str(iterations)]
    return json.loads(subprocess.check_output(command,env=env,text=True))

fixture_manifest = None
if os.environ.get('MC_TICK_BENCH_FIXTURES'):
    fixture_dir = pathlib.Path(os.environ['MC_TICK_BENCH_FIXTURES'])
    fixture_manifest = json.loads((fixture_dir / 'manifest.json').read_text())
    for name, entry in fixture_manifest['fixtures'].items():
        assert hashlib.sha256((fixture_dir / (name+'.snbt')).read_bytes()).hexdigest() == entry['snbt_sha256'], f'{name}: generated fixture changed'

metadata={'kind' :'metadata','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'host':platform.node(),'platform':platform.platform(),'cpu':cpu,'allowed_cpus':allowed,
    'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text().split('\n\n')[0],
    'fixture_manifest':fixture_manifest,'seed':a.seed,'rounds':a.rounds,'target_ms':a.target_ms,'modes':modes,'plan_modes':plan_modes,
    'piston_plan_mode':os.environ.get('MC_TICK_PISTON_PLAN','on'),
    'transition_signature_mode':os.environ.get('MC_TICK_TRANSITION_SIGNATURES','on'),
    'variants':{name:{'path':path,'sha256':hashlib.file_digest(open(path,'rb'),'sha256').hexdigest()}
                for name,path in variants.items()},
    'method':'one warmup sample per subprocess; input+stepping+recording timed; restore/setup/drop excluded',
    'build':json.loads(a.build_info.read_text()) if a.build_info else None}
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
