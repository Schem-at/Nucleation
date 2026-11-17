#!/usr/bin/env python3
"""Capture one community scenario in the pinned Minecraft 1.21.11.

Downloads nothing. Requires a new output directory and preserves logs on error.
Run on harrison-work separately from benchmarks. A structurally valid capture is
NOT a functionality/parity pass: positive and negative assertions are separate.

Scenario JSON (coordinates are relative to origin; watch bounds inclusive):
  {"format_version":1,"data_version":4671,"case":"name","negative_control":false,
   "size":[6,4,1],"origin":[0,100,0],"watch":{"min":[-2,-2,-2],"max":[7,5,2]},
   "rng_seed":0,"capture_ticks":40,
   "source":{"sha256":"<64 lowercase hex>","data_version":4671,"url":"..."},
   "fixture":[{"pos":[0,0,0],"state":"minecraft:stone","nbt":"{...}"}],
   "actions":[{"tick":1,"type":"notify_neighbors","pos":[0,1,0]}]}

States must be complete canonical descriptors with sorted properties. Omit nbt
for non-BE blocks. set_block actions require state and use ordinary flag-3 writes;
they are not player interactions. pull_lever targets an existing lever and calls
vanilla LeverBlock.pull with a null player (no player joins). Actions preserve
input order within a tick.
use_note_block calls the actual NoteBlock.useWithoutItem with a detached player.
For a different source version, explicitly set source_mode=exact_states_no_datafix.
That mode rejects block entities and preserves source.data_version; every state
must load unchanged in 1.21.11. It tests that version, not the source's version.
No DFU, entity loading, item-component capture or moving-BE capture is provided.

Optional capture_stride (positive integer, at most capture_ticks) retains tick
zero, every stride-th tick and the final tick. All ticks/actions still execute.
Sparse output declares sampled_ticks and cannot establish behavior between them.
"""
import argparse
import datetime
import json
import math
import os
import pathlib
import platform
import re
import subprocess

from run_capture12111 import (INNER_SHA256, JDK_SHA256, MAPPINGS_SHA1,
                              SERVER_SHA1, check_runtime, digest)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f'Duplicate JSON key: {key}')
        result[key] = value
    return result


def load_json(path):
    if path.stat().st_size > 16 * 1024 * 1024:
        raise ValueError('JSON input exceeds 16 MiB')
    return json.loads(path.read_text(), object_pairs_hook=unique_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def int32(value, label):
    if type(value) is not int or not -(2**31) <= value < 2**31:
        raise ValueError(f'{label} must be an int32')
    return value


def vector(value, label):
    if not isinstance(value, list) or len(value) != 3:
        raise ValueError(f'{label} must have three coordinates')
    return [int32(v, label) for v in value]


def sampled_ticks(data):
    ticks = int32(data.get('capture_ticks'), 'capture_ticks')
    stride = int32(data.get('capture_stride', 1), 'capture_stride')
    if not 1 <= ticks <= 5000:
        raise ValueError('Capture exceeds bounded simulated-tick budget')
    if not 1 <= stride <= ticks:
        raise ValueError('capture_stride must be positive and at most capture_ticks')
    result = list(range(0, ticks + 1, stride))
    if result[-1] != ticks:
        result.append(ticks)
    return result


def validate_scenario(data):
    if not isinstance(data, dict) or data.get('format_version') != 1:
        raise ValueError('Expected scenario format_version 1')
    if int32(data.get('data_version'), 'data_version') != 4671:
        raise ValueError('Only native 4671 fixtures are accepted; no datafixing')
    if not isinstance(data.get('case'), str) or not re.fullmatch(r'[a-zA-Z0-9_-]+', data['case']):
        raise ValueError('case must be a safe nonempty label')
    if type(data.get('negative_control')) is not bool:
        raise ValueError('negative_control must be boolean')
    source = data.get('source')
    if (not isinstance(source, dict) or type(source.get('data_version')) is not int
            or not re.fullmatch(r'[0-9a-f]{64}', str(source.get('sha256', '')))):
        raise ValueError('source requires original data_version and SHA-256')
    exact_states = data.get('source_mode') == 'exact_states_no_datafix'
    if data.get('source_mode') not in (None, 'exact_states_no_datafix'):
        raise ValueError('Unknown source_mode')
    if source['data_version'] != 4671 and not exact_states:
        raise ValueError('Non-native sources require explicit exact_states_no_datafix mode')
    size = vector(data.get('size'), 'size')
    origin = vector(data.get('origin'), 'origin')
    watch = data.get('watch', {})
    lower, upper = vector(watch.get('min'), 'watch.min'), vector(watch.get('max'), 'watch.max')
    volume = 1
    for i in range(3):
        if size[i] <= 0 or lower[i] > 0 or upper[i] < size[i] - 1:
            raise ValueError('Watch must contain the positive fixture size')
        extent = upper[i] - lower[i] + 1
        if not 1 <= extent <= 262144:
            raise ValueError('Invalid watch extent')
        volume *= extent
        if not -29999968 <= origin[i] + lower[i] <= origin[i] + upper[i] <= 29999967:
            raise ValueError('Watch outside bounded world coordinates')
    if not -64 <= origin[1] + lower[1] <= origin[1] + upper[1] < 320:
        raise ValueError('Watch outside overworld build height')
    ticks = int32(data.get('capture_ticks'), 'capture_ticks')
    frames = sampled_ticks(data)
    # The existing budget counts post-initial snapshots. Sparse sampling reduces
    # retained/scanned cells, not the number of simulated ticks (still <=5000).
    if volume > 262144 or volume * (len(frames) - 1) > 50000000:
        raise ValueError('Capture exceeds bounded cell/snapshot budget')
    x_chunks = ((origin[0] + upper[0]) >> 4) - ((origin[0] + lower[0]) >> 4) + 3
    z_chunks = ((origin[2] + upper[2]) >> 4) - ((origin[2] + lower[2]) >> 4) + 3
    if x_chunks * z_chunks > 1024:
        raise ValueError('Too many chunk tickets')
    int32(data.get('rng_seed'), 'rng_seed')
    fixture = data.get('fixture')
    if not isinstance(fixture, list) or not fixture:
        raise ValueError('Fixture must be a nonempty array')
    occupied = set()
    for block in fixture:
        p = vector(block.get('pos'), 'fixture.pos')
        if any(not 0 <= p[i] < size[i] for i in range(3)) or tuple(p) in occupied:
            raise ValueError('Fixture positions must be unique and inside size')
        occupied.add(tuple(p))
        if not isinstance(block.get('state'), str) or not block['state']:
            raise ValueError('Fixture requires a state descriptor')
        if 'nbt' in block and (not isinstance(block['nbt'], str) or not block['nbt'] or block['state'] == 'minecraft:air'):
            raise ValueError('Non-air BE nbt must be a nonempty SNBT string')
        if exact_states and 'nbt' in block:
            raise ValueError('Exact-state compatibility mode cannot import block entities')
    if data.get('entities') or data.get('item_entities'):
        raise ValueError('Entity loading is not implemented by this oracle')
    actions = data.get('actions')
    if not isinstance(actions, list):
        raise ValueError('actions must be an array')
    previous = 0
    for action in actions:
        tick = int32(action.get('tick'), 'action.tick')
        p = vector(action.get('pos'), 'action.pos')
        if not 1 <= tick <= ticks or tick < previous or any(not lower[i] <= p[i] <= upper[i] for i in range(3)):
            raise ValueError('Actions must be ordered, inside watch and within numbered ticks')
        previous = tick
        if action.get('type') == 'set_block':
            if not isinstance(action.get('state'), str) or not action['state']:
                raise ValueError('set_block requires a state')
        elif action.get('type') not in {'notify_neighbors', 'pull_lever', 'use_note_block'} or 'state' in action:
            raise ValueError('Unknown/invalid action')
    return volume


def validate_capture(path, scenario, scenario_sha256):
    # Captures may exceed the bounded INPUT size; parse their complete output.
    data = json.loads(path.read_text(), object_pairs_hook=unique_object)
    expected = {'mc_version': '1.21.11', 'data_version': 4671,
                'mappings_sha1': MAPPINGS_SHA1, 'server_inner_sha256': INNER_SHA256,
                'scenario_sha256': scenario_sha256, 'settle': 'in_world'}
    for key in ['format_version', 'case', 'negative_control', 'size', 'origin', 'watch',
                'rng_seed', 'capture_ticks', 'source', 'fixture', 'actions']:
        expected[key] = scenario[key]
    for key, value in expected.items():
        if data.get(key) != value:
            raise ValueError(f'Capture changed {key}')
    if data.get('source_mode') != scenario.get('source_mode'):
        raise ValueError('Capture changed source_mode')
    expected_ticks = sampled_ticks(scenario)
    if 'capture_stride' in scenario:
        if data.get('capture_stride') != scenario['capture_stride']:
            raise ValueError('Capture changed capture_stride')
        if data.get('sampled_ticks') != expected_ticks:
            raise ValueError('Capture changed sampled_ticks')
        if scenario['capture_stride'] > 1 and 'sampled tick-end' not in data.get('scope', ''):
            raise ValueError('Sparse capture must declare its sampled tick-end scope')
    elif 'capture_stride' in data or 'sampled_ticks' in data:
        raise ValueError('Capture introduced undeclared sampling')
    frames = [data['initial'], *data['snapshots']]
    if [frame['tick'] for frame in frames] != expected_ticks:
        raise ValueError('Capture does not contain the exact declared tick sequence')
    lower, upper = scenario['watch']['min'], scenario['watch']['max']
    for frame in frames:
        if frame['players'] != 0:
            raise ValueError('A player was present')
        for name, length, keys in [('blocks', 4, 3), ('inventories', 6, 4), ('comparator_outputs', 4, 3)]:
            seen = set()
            previous = None
            for row in frame[name]:
                if not isinstance(row, list) or len(row) != length:
                    raise ValueError(f'Malformed {name} row')
                p = vector(row[:3], name)
                if any(not lower[i] <= p[i] <= upper[i] for i in range(3)):
                    raise ValueError('Snapshot row outside declared watch')
                key = tuple(row[:keys])
                if key in seen:
                    raise ValueError(f'Duplicate {name} row')
                seen.add(key)
                order = (p[1], p[2], p[0], row[3] if name == 'inventories' else -1)
                if previous is not None and order < previous:
                    raise ValueError(f'Unexpected {name} storage order')
                previous = order
                if name == 'inventories' and (type(row[3]) is not int or row[3] < 0
                        or not isinstance(row[4], str) or type(row[5]) is not int or row[5] <= 0):
                    raise ValueError('Invalid occupied inventory row')
                if name == 'comparator_outputs' and (type(row[3]) is not int or not 0 <= row[3] <= 15):
                    raise ValueError('Invalid comparator output')
                if name == 'blocks' and (not isinstance(row[3], str) or row[3] == 'minecraft:air'):
                    raise ValueError('Invalid non-air state row')
    initial = {tuple(row[:3]): row[3] for row in frames[0]['blocks']}
    expected_blocks = {tuple(b['pos']): b['state'] for b in scenario['fixture'] if b['state'] != 'minecraft:air'}
    if initial != expected_blocks:
        raise ValueError('Initial blocks do not equal the input fixture')
    if scenario.get('capture_stride', 1) > 1:
        activity = data.get('final_activity', {})
        if activity.get('tick') != scenario['capture_ticks']:
            raise ValueError('Sparse capture requires final activity at the last tick')
        for name in ['block_ticks', 'fluid_ticks', 'block_events',
                     'rescheduled_block_events', 'moving_pistons_in_watch']:
            if type(activity.get(name)) is not int or activity[name] < 0:
                raise ValueError(f'Invalid final activity counter: {name}')
        moving = sum(row[3].startswith('minecraft:moving_piston[') for row in frames[-1]['blocks'])
        if activity['moving_pistons_in_watch'] != moving:
            raise ValueError('Final moving-piston count disagrees with final frame')
        if not isinstance(activity.get('scope'), str) or not activity['scope']:
            raise ValueError('Final activity must declare its scope')
    result = {'sha256': digest(path), 'ticks': scenario['capture_ticks'], 'players': 0,
            'warmup_ticks': data['warmup_ticks'], 'java': data['java_runtime_version'],
            'structural_capture_checks': 'passed', 'functional_checks': 'not performed'}
    if 'capture_stride' in scenario:
        result.update(capture_stride=scenario['capture_stride'], sampled_ticks=expected_ticks,
                      every_tick_captured=scenario['capture_stride'] == 1)
    if scenario.get('capture_stride', 1) > 1:
        result['final_activity'] = data['final_activity']
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--runtime', type=pathlib.Path, required=True)
    parser.add_argument('--jdk', type=pathlib.Path, required=True)
    parser.add_argument('--jdk-archive', type=pathlib.Path)
    parser.add_argument('--scenario', type=pathlib.Path, required=True)
    parser.add_argument('--source-file', type=pathlib.Path,
                        help='Verify these original fixture bytes against scenario source.sha256')
    parser.add_argument('--output', type=pathlib.Path, required=True)
    parser.add_argument('--timeout', type=float, default=300)
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('--timeout must be finite and positive')
    runtime, jdk, output = args.runtime.resolve(), args.jdk.resolve(), args.output.resolve()
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        parser.error('--output must be new or empty')
    scenario = load_json(args.scenario)
    cells = validate_scenario(scenario)
    source_verified = False
    if args.source_file:
        if digest(args.source_file) != scenario['source']['sha256']:
            raise ValueError('Original fixture SHA-256 mismatch')
        source_verified = True
    inner, libraries, verified, version = check_runtime(runtime)
    if args.jdk_archive and digest(args.jdk_archive) != JDK_SHA256:
        raise ValueError('Temurin JDK archive checksum mismatch')
    java, javac = jdk / 'bin/java', jdk / 'bin/javac'
    java_version = subprocess.check_output([str(java), '-version'], stderr=subprocess.STDOUT, text=True, timeout=30)
    javac_version = subprocess.check_output([str(javac), '-version'], stderr=subprocess.STDOUT, text=True, timeout=30)
    if not re.search(r'version "21\.0\.13(?:"|[.+-])', java_version) or not javac_version.startswith('javac 21.0.13'):
        raise ValueError('This pinned recipe requires Java/javac 21.0.13')
    source_dir = pathlib.Path(__file__).resolve().parent / 'src'
    sources = [source_dir / name for name in ['TraceCapture12111.java', 'CommunityCapture12111.java']]
    output.mkdir(parents=True, exist_ok=True)
    classes = output / 'classes'
    classes.mkdir()
    # Preserve exact input bytes, not a reserialization with a different hash.
    saved_scenario = output / 'scenario.json'
    saved_scenario.write_bytes(args.scenario.read_bytes())
    scenario_hash = digest(saved_scenario)
    metadata = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                'host': platform.node(), 'platform': platform.platform(),
                'sources_sha256': {str(source): digest(source) for source in sources},
                'runner_sha256': digest(pathlib.Path(__file__)),
                'shared_runner_sha256': digest(pathlib.Path(__file__).with_name('run_capture12111.py')),
                'scenario_sha256': scenario_hash, 'source': scenario['source'],
                'original_source_bytes_verified': source_verified,
                'runtime': str(runtime), 'server_sha1': SERVER_SHA1,
                'mappings_sha1': MAPPINGS_SHA1, 'verified_extracted_sha256': verified,
                'version': version, 'java_version': java_version, 'javac_version': javac_version,
                'jdk_archive_verified': bool(args.jdk_archive), 'watch_cells': cells,
                'commands': [], 'functional_checks': 'not performed'}
    if 'capture_stride' in scenario:
        metadata.update(capture_stride=scenario['capture_stride'],
                        sampled_ticks=sampled_ticks(scenario),
                        every_tick_captured=scenario['capture_stride'] == 1)

    def save():
        (output / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')

    def run(command, label):
        metadata['commands'].append({'label': label, 'argv': command})
        save()
        with (output / (label + '.log')).open('w') as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT,
                           timeout=args.timeout, check=True, cwd=output)

    try:
        run([str(javac), '-d', str(classes), *map(str, sources)], 'compile')
        capture = output / 'capture.json'
        classpath = os.pathsep.join(map(str, [classes, inner, *libraries]))
        run([str(java), '-Xmx2G', '-cp', classpath, 'org.nucleation.oracle.CommunityCapture12111',
             '--mappings', str(runtime / 'server.txt'), '--universe', str(output / 'universe'),
             '--out', str(capture), '--scenario', str(saved_scenario)], 'capture')
        metadata['capture'] = validate_capture(capture, scenario, scenario_hash)
        metadata['complete_capture'] = True
        print(f"{scenario['case']}: {scenario['capture_ticks']} ticks, zero players; structural checks passed; functional checks remain separate", flush=True)
    except Exception as error:
        metadata['complete_capture'] = False
        metadata['error'] = repr(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
