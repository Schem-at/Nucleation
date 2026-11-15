#!/usr/bin/env python3
"""Run four finite, playerless 1.21.11 captures with a supplied runtime and JDK.

Downloads nothing, changes no global configuration, and never overwrites a run.
Run separately from native benchmarks. Logs and partial metadata survive errors.
"""
import argparse
import datetime
import hashlib
import json
import pathlib
import platform
import re
import subprocess
import zipfile

SERVER_SHA1 = '64bb6d763bed0a9f1d632ec347938594144943ed'
MAPPINGS_SHA1 = '5621e9253f05fd57872bbe7f8ddf5f9a7d525955'
INNER_SHA256 = 'ec47239a8de246335e1d54f6ac319bd35641778eb4b6a6da06372840d02fcebc'
JDK_SHA256 = 'd7f5d5cf865a2902b70ff549b97586a7d3f234e016f041ab447ea740c7d66d1c'
CASES = [('comparator_drain', False), ('comparator_drain', True),
         ('piston_qc', False), ('piston_qc', True)]


def digest(path, algorithm='sha256'):
    with path.open('rb') as f:
        return hashlib.file_digest(f, algorithm).hexdigest()


def check_runtime(runtime):
    for name, expected in [('server.jar', SERVER_SHA1), ('server.txt', MAPPINGS_SHA1)]:
        actual = digest(runtime / name, 'sha1')
        if actual != expected:
            raise ValueError(f'{name}: expected SHA-1 {expected}, got {actual}')
    verified = {}
    libraries = []
    inner = None
    with zipfile.ZipFile(runtime / 'server.jar') as bundle:
        damaged = bundle.testzip()
        if damaged:
            raise ValueError(f'Bundle CRC failure: {damaged}')
        for kind in ['versions', 'libraries']:
            base = (runtime / 'inner' / 'META-INF' / kind).resolve()
            for line in bundle.read(f'META-INF/{kind}.list').decode().splitlines():
                expected, identity, relative = line.split('\t')
                artifact = (base / relative).resolve()
                if not artifact.is_relative_to(base):
                    raise ValueError(f'Artifact escapes runtime: {relative}')
                if digest(artifact) != expected:
                    raise ValueError(f'Extracted artifact SHA-256 mismatch: {artifact}')
                verified[str(artifact.relative_to(runtime))] = expected
                if kind == 'libraries':
                    libraries.append(artifact)
                elif identity == '1.21.11':
                    inner = artifact
    if inner is None or digest(inner) != INNER_SHA256:
        raise ValueError('Verified 1.21.11 inner server not found')
    with zipfile.ZipFile(inner) as jar:
        version = json.loads(jar.read('version.json'))
    if version['id'] != '1.21.11' or version['world_version'] != 4671:
        raise ValueError(f'Unexpected embedded version: {version}')
    return inner, sorted(libraries), verified, version


def validate_capture(path, scenario, negative):
    data = json.loads(path.read_text())
    expected = {'mc_version': '1.21.11', 'data_version': 4671,
                'case': scenario, 'negative_control': negative,
                'capture_ticks': 40, 'mappings_sha1': MAPPINGS_SHA1,
                'server_inner_sha256': INNER_SHA256}
    for key, value in expected.items():
        if data.get(key) != value:
            raise ValueError(f'{path.name}: unexpected {key}: {data.get(key)!r}')
    if data['initial']['tick'] != 0:
        raise ValueError('Missing tick-zero initial state')
    frames = data['snapshots']
    if [row['tick'] for row in frames] != list(range(1, 41)):
        raise ValueError('Capture does not contain exactly ticks 1 through 40')
    for row in [data['initial'], *frames]:
        if row['players'] != 0:
            raise ValueError('A player was present during capture')
    # Independent checks supplement the driver's checks, so an existing or
    # silently inert file cannot be accepted merely because the JVM exited zero.
    def count(frame, pos):
        return sum(row[5] for row in frame['inventories'] if row[:3] == pos)
    def state(frame, pos):
        return next((row[3] for row in frame['blocks'] if row[:3] == pos), 'minecraft:air')
    if scenario == 'comparator_drain':
        if count(data['initial'], [2, 2, 0]) != 3:
            raise ValueError('Initial inventory does not contain three items')
        if negative:
            if any(count(row, [2, 2, 0]) != 3 or count(row, [2, 1, 0]) != 0 for row in frames):
                raise ValueError('Locked hopper negative control transferred items')
        else:
            if count(frames[-1], [2, 2, 0]) != 0 or count(frames[-1], [2, 1, 0]) != 3:
                raise ValueError('Barrel did not drain into hopper')
            outputs = [row['comparator_outputs'] for row in frames]
            if not any([3, 2, 0, 1] in row for row in outputs) or outputs[-1] != [[3, 2, 0, 0]]:
                raise ValueError('Comparator did not rise and fall')
    elif negative:
        if any('extended=true' in state(row, [2, 1, 0]) for row in frames):
            raise ValueError('Unpowered piston negative control extended')
    else:
        if not all('extended=true' in state(frames[tick - 1], [2, 1, 0]) for tick in [1, 10, 11]):
            raise ValueError('QC extension/hold boundary did not match')
        if 'extended=false' not in state(frames[-1], [2, 1, 0]):
            raise ValueError('QC piston did not retract')
    return {'sha256': digest(path), 'ticks': 40, 'players': 0,
            'warmup_ticks': data['warmup_ticks'], 'java': data['java_runtime_version']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=pathlib.Path, required=True)
    parser.add_argument('--jdk', type=pathlib.Path, required=True)
    parser.add_argument('--jdk-archive', type=pathlib.Path,
                        help='Optionally verify the supplied Temurin 21.0.13+9 tarball as well')
    parser.add_argument('--output', type=pathlib.Path, required=True,
                        help='New or empty directory; no existing captures/worlds are overwritten')
    parser.add_argument('--timeout', type=float, default=180,
                        help='Maximum seconds for compilation and each capture separately')
    args = parser.parse_args()
    runtime, jdk, output = args.runtime.resolve(), args.jdk.resolve(), args.output.resolve()
    if args.timeout <= 0:
        parser.error('--timeout must be positive')
    if output.exists() and any(output.iterdir()):
        parser.error('--output must be new or empty')
    source = pathlib.Path(__file__).resolve().parent / 'src' / 'TraceCapture12111.java'
    inner, libraries, verified, version = check_runtime(runtime)
    if args.jdk_archive and digest(args.jdk_archive) != JDK_SHA256:
        raise ValueError('Temurin JDK archive checksum mismatch')
    java, javac = jdk / 'bin/java', jdk / 'bin/javac'
    java_version = subprocess.check_output([str(java), '-version'], stderr=subprocess.STDOUT, text=True, timeout=30)
    javac_version = subprocess.check_output([str(javac), '-version'], stderr=subprocess.STDOUT, text=True, timeout=30)
    if not re.search(r'version "21\.0\.13(?:"|[.+-])', java_version) or not javac_version.startswith('javac 21.0.13'):
        raise ValueError('This pinned capture recipe requires Java/javac 21.0.13')
    output.mkdir(parents=True, exist_ok=True)
    classes = output / 'classes'
    classes.mkdir()
    metadata = {'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                'host': platform.node(), 'platform': platform.platform(),
                'source': str(source), 'source_sha256': digest(source),
                'runner_sha256': digest(pathlib.Path(__file__)),
                'runtime': str(runtime), 'server_sha1': SERVER_SHA1,
                'mappings_sha1': MAPPINGS_SHA1, 'verified_extracted_sha256': verified,
                'version': version, 'java_version': java_version, 'javac_version': javac_version,
                'jdk_archive_verified': bool(args.jdk_archive), 'commands': [], 'captures': {}}
    def save():
        (output / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    def run(command, label):
        metadata['commands'].append({'label': label, 'argv': command})
        save()
        with (output / (label + '.log')).open('w') as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT,
                           timeout=args.timeout, check=True, cwd=output)
    try:
        run([str(javac), '-d', str(classes), str(source)], 'compile')
        classpath = ':'.join(map(str, [classes, inner, *libraries]))
        for scenario, negative in CASES:
            label = scenario + ('_negative' if negative else '')
            capture = output / (label + '.json')
            command = [str(java), '-Xmx2G', '-cp', classpath,
                       'org.nucleation.oracle.TraceCapture12111',
                       '--mappings', str(runtime / 'server.txt'),
                       '--universe', str(output / ('universe-' + label)),
                       '--out', str(capture), '--case', scenario, '--ticks', '40']
            if negative:
                command.append('--negative-control')
            run(command, label)
            metadata['captures'][label] = validate_capture(capture, scenario, negative)
            save()
            print(f'{label}: 40 ticks, zero players, control checks passed', flush=True)
        metadata['complete'] = True
    except Exception as error:
        metadata['complete'] = False
        metadata['error'] = repr(error)
        raise
    finally:
        save()


if __name__ == '__main__':
    main()
