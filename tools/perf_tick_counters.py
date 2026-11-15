#!/usr/bin/env python3
"""Linux x86-64 perf_event counters for complete benchmark subprocesses.

Includes loading, warmup and measured cycles. These are explanatory counters,
not replacements for the tick-only paired timing runner. No sampling or root.
"""
import argparse, ctypes, fcntl, hashlib, json, os, pathlib, platform, random, signal, struct, tempfile
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--variant',action='append',required=True)
p.add_argument('--case',default='flyer');p.add_argument('--iterations',type=int,default=512)
p.add_argument('--rounds',type=int,default=9);p.add_argument('--cpu',type=int,default=0)
p.add_argument('--output',type=pathlib.Path,required=True)
a=p.parse_args()
assert platform.system()=='Linux' and platform.machine()=='x86_64'
assert a.cpu in os.sched_getaffinity(0)
assert not any(k.startswith('MC_TICK_TRACE_') for k in os.environ)
assert all(os.environ.get(k)!='shadow' for k in ['MC_TICK_WIRE_PLAN','MC_TICK_PISTON_PLAN'])
variants=dict(x.split('=',1) for x in a.variant)
lib=ctypes.CDLL(None,use_errno=True)
metrics={'cycles':0,'instructions':1,'cache_misses':3,'branches':4,'branch_misses':5}
def event(pid,config):
    attr=ctypes.create_string_buffer(128)
    # disabled, exclude_kernel, exclude_hv; read value/time_enabled/time_running.
    struct.pack_into('IIQQQQQ',attr,0,0,128,config,0,0,3,(1<<0)|(1<<5)|(1<<6))
    fd=lib.syscall(298,ctypes.byref(attr),pid,-1,-1,8) # PERF_FLAG_FD_CLOEXEC
    if fd<0: raise OSError(ctypes.get_errno(),os.strerror(ctypes.get_errno()))
    return fd

def measure(path):
    read_gate,write_gate=os.pipe()
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        pid=os.fork()
        if pid==0:
            try:
                os.close(write_gate);os.sched_setaffinity(0,{a.cpu})
                os.dup2(stdout.fileno(),1);os.dup2(stderr.fileno(),2)
                os.read(read_gate,1);os.close(read_gate)
                os.execve(path,[path,a.case,'1',str(a.iterations)],os.environ)
            finally: os._exit(127)
        os.close(read_gate)
        fds={}
        try:
            for name,config in metrics.items():fds[name]=event(pid,config)
            for fd in fds.values():fcntl.ioctl(fd,0x2403,0);fcntl.ioctl(fd,0x2400,0)
            os.write(write_gate,b'1');os.close(write_gate);write_gate=-1
            _,status=os.waitpid(pid,0);pid=0
            counters={}
            for name,fd in fds.items():
                fcntl.ioctl(fd,0x2401,0)
                value,enabled,running=struct.unpack('QQQ',os.read(fd,24))
                assert running>0
                counters[name]={'value':value,'enabled_ns':enabled,'running_ns':running,
                    'scaled':value*enabled/running}
            stdout.seek(0);stderr.seek(0)
            err=stderr.read().decode();assert os.waitstatus_to_exitcode(status)==0,err
            assert not err,err
            return {'counters':counters,'benchmark':json.loads(stdout.read())}
        finally:
            if write_gate>=0:os.close(write_gate)
            if pid:os.kill(pid,signal.SIGKILL);os.waitpid(pid,0)
            for fd in fds.values():os.close(fd)

a.output.parent.mkdir(parents=True,exist_ok=True)
rng=random.Random(20261114)
with a.output.open('w') as f:
    f.write(json.dumps({'kind':'metadata','host':platform.node(),'platform':platform.platform(),
        'cpu':a.cpu,'case':a.case,'rounds':a.rounds,'iterations':a.iterations,'seed':20261114,
        'method':'perf_event_open: userspace-only complete subprocess; no sampling; scaled counters if multiplexed',
        'variants':{k:{'path':v,'sha256':hashlib.sha256(pathlib.Path(v).read_bytes()).hexdigest()} for k,v in variants.items()}})+'\n')
    for r in range(a.rounds):
        order=list(variants);rng.shuffle(order)
        for name in order:
            result=measure(variants[name]);f.write(json.dumps({'kind':'sample','round':r,'variant':name,**result})+'\n');f.flush()
        print('Counter round',r+1,'/',a.rounds,flush=True)
