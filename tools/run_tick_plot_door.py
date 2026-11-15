#!/usr/bin/env python3
"""Replay an exported door's recorded control protocol with installed Nucleation.

pip install nucleation==0.12.3
python run_tick_plot_door.py path/to/part.run.json

Completion establishes executable inputs and expected control transitions, not
aperture correctness or new vanilla parity. Use the verified adapter for that.
"""
import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import time


def run(protocol_path, source=None):
    import nucleation as n
    protocol_path = Path(protocol_path)
    spec = json.loads(protocol_path.read_bytes())
    source = Path(source) if source else protocol_path.with_name(protocol_path.name.removesuffix('.run.json') + '.litematic')
    data = source.read_bytes()
    if hashlib.sha256(data).hexdigest() != spec['source_sha256']:
        raise ValueError('Schematic differs from the source used by this protocol')
    sc = n.Schematic.from_data(data)
    origin = json.loads(sc.bounding_box_json())[:3]
    if origin != spec['hash_origin'] or sc.mc_version() != spec['source_data_version']:
        raise ValueError('Saved origin/data version differs from the recorded protocol')
    if spec['settle'] != 'in_world':
        raise ValueError('Unsupported protocol settle mode')
    sim = n.TickSimulation.from_schematic(sc, n.TickSettleMode.InWorld, *origin, '')
    sim.set_rng_seed(spec['rng_seed'])
    changes = 0
    def step():
        nonlocal changes
        before = sim.tick_count(); sim.step()
        if sim.tick_count() != before + 1:
            raise RuntimeError('Engine failed to advance one tick')
        changes += sim.changes_count()
        if not sim.clear_changes(): raise RuntimeError('Could not drain block recording')
    started = time.perf_counter()
    for _ in range(spec['setup_ticks']): step()
    actions = {}
    for action in spec['actions']:
        tick = action['before_elapsed_tick']
        if not 0 <= tick < spec['trial_ticks'] or tick in actions:
            raise ValueError('Invalid or duplicate control time')
        actions[tick] = action
    for tick in range(spec['trial_ticks']):
        if tick in actions:
            action = actions[tick]
            pos = [a-b for a,b in zip(action['position'], origin)]
            before = sim.get_block(*pos)
            if before != action['from']:
                raise AssertionError(f'Control before tick {tick}: {before} != {action["from"]}')
            sim.use_block(*pos)
            after = sim.get_block(*pos)
            if after != action['to']:
                raise AssertionError(f'Control after tick {tick}: {after} != {action["to"]}')
        step()
    return {'status': 'protocol_completed', 'source_sha256': spec['source_sha256'],
            'package_version': importlib.metadata.version('nucleation'), 'evidence_tier': spec['evidence_tier'],
            'ticks': sim.tick_count(), 'control_uses': len(actions), 'block_changes': changes,
            'elapsed_seconds_including_python_checks': time.perf_counter()-started,
            'scope': 'Fresh native package load, stepping and recorded control transitions. Not a performance benchmark or independent door/vanilla proof.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('protocol', type=Path)
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    print(json.dumps(run(args.protocol, args.source)))

if __name__ == '__main__': main()
