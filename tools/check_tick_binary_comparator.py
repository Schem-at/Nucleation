#!/usr/bin/env python3
"""Generate and check the pinned OTHERCOMB017 native-4671 comparator workload.

generate --fixture-json OTHERCOMB017.json --source-file original.schem
         --scenario-out scenario.json --checks-out checks.json
validate --fixture-json OTHERCOMB017.json --source-file original.schem
         --scenario scenario.json --capture capture.json --output result.json

The fixture JSON is produced by export_tick_oracle_fixture. Generation preserves
its raw block/BE data. Forty 24-game-tick phases use actual lever pulls before
the numbered tick. Validation derives expectations from Python integer ==/< />,
not from simulator results or the optional generated checks file. No Java/Rust
execution is performed here. A pass covers these endpoints, not exhaustive
8-bit functionality, minimal latency, transient states or simulator parity.
"""
import argparse
import copy
import hashlib
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / 'gametest'))
from run_community_capture12111 import (load_json, validate_capture,
                                      validate_scenario)

SOURCE_SHA256 = '92d44580dc1539903b56ad08a8b1be89d70de602328d5fe36ef589743c3a3db3'
SOURCE_URL = ('https://github.com/mattbatwings/Logical-Redstone-Archive/tree/'
              '6d054aa0a87d8f58f672625b3e6d0e727bd97016/Archive/'
              'OTHERCOMB_other-combinational/OTHERCOMB017_8_bit_Binary_Comparator')
PHASE_TICKS = 24
OUTPUTS = {'equal': [0, 15, 0], 'less': [3, 15, 0], 'greater': [4, 15, 3]}
LEVER_OFF = 'minecraft:lever[face=wall,facing=south,powered=false]'
LEVER_ON = LEVER_OFF.replace('powered=false', 'powered=true')


def digest(path):
    with pathlib.Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def vectors():
    result = [(0, 0), (255, 0), (255, 255), (0, 255), (0, 0)]
    for bit in range(8):
        value = 1 << bit
        result.extend([(value, 0), (value, value), (0, value), (0, 0)])
    result.extend([(1, 128), (128, 1), (0, 0)])
    assert len(result) == 40
    return result


def expectations():
    return [{'tick': (phase + 1) * PHASE_TICKS, 'a': a, 'b': b,
             'equal': a == b, 'less': a < b, 'greater': a > b}
            for phase, (a, b) in enumerate(vectors())]


def actions():
    result = []
    previous = (0, 0)
    for phase, values in enumerate(vectors()):
        for column, (old, new) in enumerate(zip(previous, values)):
            for bit in range(8):
                if (old ^ new) & (1 << bit):
                    result.append({'tick': phase * PHASE_TICKS + 1,
                                   'type': 'pull_lever',
                                   'pos': [column * 3, 1 + 2 * bit, 10]})
        previous = values
    return result


def fixture_and_source(export_path, source_path):
    actual = digest(source_path)
    if actual != SOURCE_SHA256:
        raise ValueError(f'Expected pinned OTHERCOMB017 SHA-256 {SOURCE_SHA256}, got {actual}')
    exported = load_json(export_path)
    if (exported.get('format_version') != 1 or exported.get('data_version') != 4671
            or exported.get('size') != [5, 17, 12] or exported.get('entity_count') != 0):
        raise ValueError('Expected the native-4671, entity-free OTHERCOMB017 export')
    fixture = exported.get('fixture')
    if not isinstance(fixture, list) or len(fixture) != 272:
        raise ValueError('Expected 272 exported fixture blocks')
    states = {tuple(block['pos']): block['state'] for block in fixture}
    if len(states) != len(fixture):
        raise ValueError('Duplicate fixture position')
    for column in [0, 3]:
        for bit in range(8):
            if states.get((column, 1 + 2 * bit, 10)) != LEVER_OFF:
                raise ValueError('Input lever position/orientation/saved power changed')
    for name, pos in OUTPUTS.items():
        expected = f"minecraft:redstone_lamp[lit={'true' if name == 'equal' else 'false'}]"
        if states.get(tuple(pos)) != expected:
            raise ValueError(f'Saved {name} output lamp changed')
    return copy.deepcopy(fixture), actual


def scenario_for(export_path, source_path):
    fixture, source_hash = fixture_and_source(export_path, source_path)
    scenario = {'format_version': 1, 'data_version': 4671, 'size': [5, 17, 12],
                'fixture': fixture, 'case': 'OTHERCOMB017', 'negative_control': False,
                'origin': [0, 100, 0], 'watch': {'min': [-2, -2, -2], 'max': [6, 18, 13]},
                'rng_seed': 0, 'capture_ticks': PHASE_TICKS * len(vectors()),
                'actions': actions(), 'source': {'sha256': source_hash,
                'data_version': 4671, 'url': SOURCE_URL}}
    validate_scenario(scenario)
    return scenario


def save_new(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def verify(export_path, source_path, scenario_path, capture_path):
    expected_scenario = scenario_for(export_path, source_path)
    scenario = load_json(scenario_path)
    if scenario != expected_scenario:
        raise ValueError('Scenario differs from this fixed generated protocol or exported fixture')
    structural = validate_capture(capture_path, scenario, digest(scenario_path))
    capture = json.loads(capture_path.read_text())
    frames = {frame['tick']: frame for frame in capture['snapshots']}
    observations, failures = [], []
    input_checks = 0
    for expected in expectations():
        tick = expected['tick']
        states = {tuple(row[:3]): row[3] for row in frames[tick]['blocks']}
        # Confirm actual input levers, so an absent pull or incorrect bit mapping
        # cannot masquerade as a successful arithmetic output assertion.
        for column, value in zip([0, 3], [expected['a'], expected['b']]):
            for bit in range(8):
                pos = (column, 1 + 2 * bit, 10)
                wanted = LEVER_ON if value & (1 << bit) else LEVER_OFF
                input_checks += 1
                if states.get(pos) != wanted:
                    failures.append({'tick': tick, 'kind': 'input', 'pos': list(pos),
                                     'expected': wanted, 'actual': states.get(pos)})
        for name, pos in OUTPUTS.items():
            state = states.get(tuple(pos))
            if state == 'minecraft:redstone_lamp[lit=true]':
                actual = True
            elif state == 'minecraft:redstone_lamp[lit=false]':
                actual = False
            else:
                actual = None
            row = {'tick': tick, 'a': expected['a'], 'b': expected['b'],
                   'output': name, 'pos': pos, 'expected': expected[name],
                   'actual': actual, 'state': state, 'passed': actual is expected[name]}
            observations.append(row)
            if not row['passed']:
                failures.append({'kind': 'output', **row})
    return {'schema_version': 1, 'case': 'OTHERCOMB017',
            'status': 'functional_pass' if not failures else 'functional_fail',
            'reference': 'Minecraft Java 1.21.11 GameTestServer capture',
            'source_sha256': digest(source_path), 'fixture_export_sha256': digest(export_path),
            'scenario_sha256': digest(scenario_path), 'capture_sha256': digest(capture_path),
            'validator_sha256': digest(pathlib.Path(__file__)),
            'structural_capture_checks': structural['structural_capture_checks'],
            'phase_ticks': PHASE_TICKS, 'phases': len(vectors()), 'capture_ticks': 960,
            'input_actions': len(actions()), 'output_assertions': len(observations),
            'output_assertions_passed': sum(row['passed'] for row in observations),
            'input_state_assertions': input_checks,
            'input_state_assertions_passed': input_checks - sum(row['kind'] == 'input' for row in failures),
            'players': 0, 'ports': OUTPUTS,
            'bit_mapping': {'A': '[0, 1+2*i, 10]', 'B': '[3, 1+2*i, 10]',
                            'bit_weight': '2**i for i=0..7; bottom-to-top significance'},
            'assumptions': [
                'Output meanings are read from source signs: A=B, A<B and A>B.',
                'Inputs are unsigned 8-bit values; bottom-to-top significance is an explicit test hypothesis.',
                'The asymmetric 1-vs-128 cases discriminate end-to-end bit reversal. These 40 vectors do not exhaustively establish every bit-order/logic case.',
                'Output assertions read lamps at each 24-game-tick phase end; they do not measure minimal circuit latency or transient electrical behavior.',
                'Actual vanilla LeverBlock.pull is used, with a null player; this is not lever block replacement.',
                'No claim about full entity/component/moving-BE behavior or intra-tick notification order.'
            ],
            'scope': 'Source-independent integer expectations for 40 reference phase endpoints, 120 output assertions. Not exhaustive 65,536-pair testing and not simulator parity.',
            'checks': expectations(), 'observations': observations,
            'failures': failures, 'first_failure': failures[0] if failures else None}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    modes = parser.add_subparsers(dest='mode', required=True)
    generate = modes.add_parser('generate', help='Produce fixed scenario and independent endpoint expectations')
    validate = modes.add_parser('validate', help='Check a fresh vanilla capture without changing expectations')
    for subparser in [generate, validate]:
        subparser.add_argument('--fixture-json', type=pathlib.Path, required=True)
        subparser.add_argument('--source-file', type=pathlib.Path, required=True)
    generate.add_argument('--scenario-out', type=pathlib.Path, required=True)
    generate.add_argument('--checks-out', type=pathlib.Path, required=True)
    validate.add_argument('--scenario', type=pathlib.Path, required=True)
    validate.add_argument('--capture', type=pathlib.Path, required=True)
    validate.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    if args.mode == 'generate':
        if args.scenario_out.exists() or args.checks_out.exists():
            parser.error('Use new output paths; existing evidence is never overwritten')
        scenario = scenario_for(args.fixture_json, args.source_file)
        save_new(args.scenario_out, scenario)
        save_new(args.checks_out, expectations())
        print('Generated 40 phases, 960 ticks, 72 lever pulls and 120 independent output expectations')
    else:
        if args.output.exists():
            parser.error('--output must be new; existing evidence is never overwritten')
        try:
            result = verify(args.fixture_json, args.source_file, args.scenario, args.capture)
        except Exception as error:
            save_new(args.output, {'schema_version': 1, 'case': 'OTHERCOMB017',
                                  'status': 'invalid_capture_or_protocol', 'error': str(error),
                                  'validator_sha256': digest(pathlib.Path(__file__))})
            raise
        save_new(args.output, result)
        print(f"{result['status']}: {result['output_assertions_passed']}/120 outputs, "
              f"{result['input_state_assertions_passed']}/640 input states")
        if result['status'] != 'functional_pass':
            raise SystemExit(1)


if __name__ == '__main__':
    main()
