#!/usr/bin/env python3
"""Check the saved execution-next evidence without compiling or timing anything."""
import collections
import hashlib
import json
import pathlib
import re
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / 'docs/performance/execution-next'


def read(path):
    return json.loads(path.read_text())


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_counts(path):
    counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', path.read_text())
    assert counts, path
    passed, failed, ignored = map(sum, zip(*(map(int, row) for row in counts)))
    assert failed == 0, path
    return {'passed': passed, 'ignored': ignored}


audit = {'source_commit': '0b8faf732ddc2317eef320af97330b8c4701e082'}
for stage in ['first', 'integrated', 'revised']:
    directory = REPORT / 'validation' / stage
    manifest = read(directory / 'manifest.json')
    archive = directory / 'sources.tar.gz'
    if 'source_archive_sha256' in manifest:
        assert sha(archive) == manifest['source_archive_sha256'], stage
    with tarfile.open(archive) as tar:
        for name, expected in manifest['source_sha256'].items():
            assert hashlib.sha256(tar.extractfile(name).read()).hexdigest() == expected, (stage, name)
            if stage == 'revised':
                assert sha(ROOT / name) == expected, ('current source', name)
    audit[stage + '_source_files'] = len(manifest['source_sha256'])

audit['engine_all_features'] = test_counts(REPORT / 'validation/checks/all-features.log')
audit['bridge'] = test_counts(REPORT / 'validation/final-checks/bridge.log')
assert audit['engine_all_features'] == {'passed': 470, 'ignored': 7}
assert audit['bridge'] == {'passed': 30, 'ignored': 0}
assert 'Finished `dev` profile' in (REPORT / 'validation/final-checks/wasm.log').read_text()
audit['wasm'] = 'passed'
audit['differential'] = {}
for suffix, expected in [('verify', 15), ('fuzz', 56)]:
    reference = lines(REPORT / f'validation/first/reference-{suffix}.jsonl')
    assert len(reference) == expected
    for path in sorted((REPORT / 'validation').rglob(f'*-{suffix}.jsonl')):
        assert lines(path) == reference, path
        audit['differential'][str(path.relative_to(REPORT))] = expected
assert len(audit['differential']) == 27

provenance = read(REPORT / 'vanilla-provenance.json')
for key in ['driver', 'runner']:
    assert sha(ROOT / provenance[key + '_source']) == provenance[key + '_source_sha256'], key
captures = ROOT / 'crates/mc-tick/tests/corpus/vanilla12111_execution_next'
assert len(list(captures.glob('*.json'))) == len(provenance['captures']) == 4
for name, entry in provenance['captures'].items():
    assert sha(ROOT / name) == entry['sha256'], name
for path in sorted(captures.glob('*.json')):
    original = read(path)
    repeated = read(REPORT / 'validation/vanilla/reproduction' / path.name)
    for key in original:
        if key != 'warmup_ticks':
            assert original[key] == repeated[key], (path.name, key)
    assert len(original['snapshots']) == 40, path
audit['vanilla'] = {'captures': 4, 'ticks_per_capture': 40, 'reproduction_matches': True}
diagnostics = read(REPORT / 'validation/final/diagnostics.json')
assert diagnostics['reference'] == diagnostics['selected']
assert diagnostics['reference']['lines'] == 17172
audit['diagnostics'] = diagnostics

raw = lines(REPORT / 'results/final-timings.jsonl')
metadata = raw[0]
samples = [row for row in raw if row['kind'] == 'sample']
calibrations = {row['case']: row for row in raw if row['kind'] == 'calibration'}
assert len(samples) == 1680 and len(calibrations) == 16
groups = collections.defaultdict(list)
for row in samples:
    assert row['iterations'] == calibrations[row['case']]['iterations']
    assert len(row['samples_ns']) == len(row['samples_cpu_ns']) == 1
    groups[row['case'], row['variant']].append(row['round'])
assert len(groups) == 80
assert all(sorted(rounds) == list(range(21)) for rounds in groups.values())
assert metadata['seed'] == 20261114 and metadata['cpu'] == 0
assert metadata['rounds'] == 21 and metadata['target_ms'] == 150
assert metadata['build'] == read(REPORT / 'metadata/final-builds.json')
build = metadata['build']
expected = {
    'previous': build['pilot-builds.json']['first']['variants']['next_reference']['sha256'],
    'selected': build['revised-builds.json']['revised']['variants']['next_view_lazy']['sha256'],
}
for variant, name in [('previous_pgo', 'reference'), ('selected_pgo', 'view-lazy'), ('feature_free_pgo', 'feature-free')]:
    expected[variant] = build[f'pgo-{name}.json']['builds']['pgo']['sha256']
assert {key: value['sha256'] for key, value in metadata['variants'].items()} == expected
audit['native'] = {'samples': len(samples), 'cases': len(calibrations), 'variants': len(expected),
                   'rounds': 21, 'matching_iterations_and_binary_hashes': True}
output = REPORT / 'validation/final-artifact-audit.json'
output.write_text(json.dumps(audit, indent=2) + '\n')
print('Verified source snapshots, 500 tests, 27 trace files, vanilla reproduction, and 1680 native samples.')
