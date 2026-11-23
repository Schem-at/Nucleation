#!/usr/bin/env python3
"""Compare an original schematic in the installed Python engine to a vanilla capture.

Run on harrison-work. Keeps source data version and origin, checks the initial
state, then compares every captured tick-end block including air. Supports real
lever/note-block uses and explicit block writes. This is not a functional oracle.
"""
import argparse
import hashlib
import importlib.metadata
import json
import pathlib
import platform


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=pathlib.Path, required=True)
    parser.add_argument('--capture', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    import nucleation as n
    data = args.source.read_bytes()
    capture_bytes = args.capture.read_bytes()
    reference = json.loads(capture_bytes)
    assert hashlib.sha256(data).hexdigest() == reference['source']['sha256']
    schematic = n.Schematic.from_data(data)
    assert schematic.mc_version() == reference['source']['data_version']
    sim = n.TickSimulation.from_schematic(schematic, n.TickSettleMode.InWorld,
                                          *reference['origin'], '')
    sim.set_rng_seed(reference['rng_seed'])
    lower, upper = reference['watch']['min'], reference['watch']['max']

    def watched(p):
        return all(lo <= v <= hi for lo, v, hi in zip(lower, p, upper))

    def native():
        return {tuple(b['pos']): b['state'] for b in json.loads(sim.world_snapshot_json())
                if watched(b['pos'])}

    def vanilla(frame):
        assert frame['players'] == 0
        return {tuple(row[:3]): row[3] for row in frame['blocks']}

    def differences(a, b):
        return [p for p in a.keys() | b.keys() if a.get(p, 'minecraft:air') != b.get(p, 'minecraft:air')]

    previous_actual, previous_expected = native(), vanilla(reference['initial'])
    assert previous_actual == previous_expected, 'Initial schematic differs from vanilla capture'
    actions = {}
    for action in reference['actions']:
        actions.setdefault(action['tick'], []).append(action)
    first = None
    frames = []
    for tick, expected_frame in enumerate(reference['snapshots'], 1):
        assert expected_frame['tick'] == tick
        for action in actions.get(tick, []):
            if action['type'] in ('pull_lever', 'use_note_block'):
                sim.use_block(*action['pos'])
            elif action['type'] == 'set_block':
                sim.place_block(*action['pos'], action['state'])
            else:
                raise ValueError('Unsupported input action: ' + action['type'])
        sim.step()
        actual, expected = native(), vanilla(expected_frame)
        diff = sorted(differences(actual, expected))
        if diff and first is None:
            first = {'tick': tick, 'positions': len(diff), 'examples': [
                {'pos': p, 'vanilla': expected.get(p, 'minecraft:air'),
                 'nucleation': actual.get(p, 'minecraft:air')} for p in diff[:12]]}
        frames.append({'tick': tick, 'mismatching_cells': len(diff),
                       'vanilla_changes': len(differences(previous_expected, expected)),
                       'nucleation_changes': len(differences(previous_actual, actual))})
        previous_actual, previous_expected = actual, expected
        sim.clear_changes()
    result = {'status': 'mismatch' if first else 'pass',
              'host': platform.node(), 'package': importlib.metadata.version('nucleation'),
              'source_sha256': reference['source']['sha256'],
              'source_data_version': schematic.mc_version(),
              'reference_version': reference['mc_version'],
              'capture_sha256': hashlib.sha256(capture_bytes).hexdigest(),
              'initial_state_matches': True, 'ticks': len(frames),
              'first_divergence': first, 'frames': frames,
              'scope': 'Tick-end blocks within capture watch box, including air; no entity or intra-tick equality.'}
    for prefix in ('vanilla', 'nucleation'):
        key = prefix + '_changes'
        result[prefix + '_tick_end_changes'] = sum(f[key] for f in frames)
        result[prefix + '_last_change_tick'] = max((f['tick'] for f in frames if f[key]), default=None)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({k: v for k, v in result.items() if k != 'frames'}, indent=2))
    return int(first is not None)


if __name__ == '__main__':
    raise SystemExit(main())
