#!/usr/bin/env python3
"""Deterministic arithmetic expectations; no simulation results are consulted."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'circuit-adapters'

def vectors():
    pairs = [(0,0),(0,255),(255,0),(255,255),(127,1),(128,128),
             (85,170),(170,85),(85,85),(170,170),(15,1),(240,16)]
    for bit in range(8):
        pairs.extend([(1 << bit,0),(0,1 << bit),((1 << (bit+1))-1,1)])
    rng = 0x4c524135
    for _ in range(128):
        rng = (rng * 1664525 + 1013904223) & 0xffffffff
        a = rng >> 24
        rng = (rng * 1664525 + 1013904223) & 0xffffffff
        pairs.append((a,rng >> 24))
    return [{'inputs': {'a': a, 'b': b, 'cin': cin},
             'expected': {'sum': (a+b+cin)&255, 'cout': a+b+cin>255}}
            for a,b in pairs for cin in [False, True]]

def generate():
    yield 'add005/suite.json', {'schema_version':1,'adapter':'add005',
          'oracle':'Independent unsigned arithmetic: total=a+b+cin; sum=total&255; cout=total>255. 328 fixed vectors, not exhaustive.',
          'vectors':vectors()}
    pairs = [(0,0),(255,0),(255,255),(0,255),(0,0)]
    for bit in range(8): pairs.extend([(1<<bit,0),(1<<bit,1<<bit),(0,1<<bit),(0,0)])
    pairs.extend([(1,128),(128,1),(0,0)])
    yield 'othercomb017/suite.json', {'schema_version':1,'adapter':'othercomb017',
          'oracle':'Independent Python integer ==, <, >; 40 fixed phase endpoints, not exhaustive.',
          'vectors':[{'inputs':{'a':a,'b':b},'expected':{'equal':a==b,'less':a<b,'greater':a>b}} for a,b in pairs]}

if __name__ == '__main__':
    p = argparse.ArgumentParser(); p.add_argument('--check', action='store_true'); a=p.parse_args()
    for name, value in generate():
        path=ROOT/name
        data=json.dumps(value,indent=2)+'\n'
        if a.check:
            if path.read_text()!=data: raise SystemExit(f'stale suite: {path}')
        else:
            path.parent.mkdir(parents=True,exist_ok=True); path.write_text(data)
