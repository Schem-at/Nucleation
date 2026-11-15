#!/usr/bin/env python3
"""Verify saved workload-specialization evidence; no compilation or timing."""
import collections
import hashlib
import json
import pathlib
import re
import subprocess
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / 'docs/performance/workload-specialization'


def read(path):
    return json.loads(path.read_text())


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def counts(path):
    rows = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', path.read_text())
    assert rows, path
    passed, failed, ignored = map(sum, zip(*(map(int, row) for row in rows)))
    assert failed == 0, path
    return {'passed': passed, 'ignored': ignored}


meta = read(REPORT / 'metadata/final-builds.json')
audit = {'master_commit': meta['master_overlay']['master_commit'],
         'candidate_commit': meta['final_source']['commit'], 'source_archives': {}}
assert audit['master_commit'] == 'ee89008d085a8fb0088851dcac7221ab94d2b91c'
for stage in ['master', 'candidates', 'certificate', 'signatures']:
    directory = REPORT / 'validation' / stage
    manifest = read(directory / 'manifest.json')
    archive = directory / 'sources.tar.gz'
    assert sha(archive) == manifest['source_archive_sha256'], stage
    with tarfile.open(archive) as tar:
        for name, expected in manifest['source_sha256'].items():
            assert hashlib.sha256(tar.extractfile(name).read()).hexdigest() == expected, (stage, name)
    audit['source_archives'][stage] = len(manifest['source_sha256'])
master = read(REPORT / 'validation/master/manifest.json')
for name, expected in meta['master_overlay']['production_source_sha256'].items():
    original = subprocess.check_output(['git', 'show', f"{audit['master_commit']}:{name}"], cwd=ROOT)
    assert hashlib.sha256(original).hexdigest() == expected, ('master Git source', name)
    if name.startswith('crates/mc-tick/'):
        assert master['source_sha256'][name] == expected, name
final_archive = REPORT / 'metadata/final-sources.tar.gz'
assert sha(final_archive) == meta['final_source']['archive_sha256']
with tarfile.open(final_archive) as tar:
    for name, expected in meta['final_source']['files'].items():
        assert sha(ROOT / name) == expected, ('current source', name)
        assert hashlib.sha256(tar.extractfile(name).read()).hexdigest() == expected, name
audit['current_source_files'] = len(meta['final_source']['files'])

audit['tests'] = counts(REPORT / 'validation/final-checks/all-features.log')
audit['bridge'] = counts(REPORT / 'validation/final-checks/bridge.log')
assert audit['tests'] == {'passed': 481, 'ignored': 8}
assert audit['bridge'] == {'passed': 30, 'ignored': 0}
assert 'Finished `dev` profile' in (REPORT / 'validation/final-checks/wasm.log').read_text()
audit['wasm'] = 'passed'
audit['standalone_overlapping_tests'] = {
    name: counts(REPORT / f'validation/standalone/{name}.log')
    for name in ['moving', 'logic']
}
assert audit['standalone_overlapping_tests'] == {
    'moving': {'passed': 249, 'ignored': 0},
    'logic': {'passed': 265, 'ignored': 0},
}
audit['traces'] = {}
reference = {suffix: lines(REPORT / f'validation/master/reference-{suffix}.jsonl')
             for suffix in ['verify', 'logic-verify', 'fuzz']}
assert [len(reference[k]) for k in ['verify', 'logic-verify', 'fuzz']] == [14, 2, 56]
for path in sorted((REPORT / 'validation').rglob('*-verify.jsonl')):
    suffix = 'logic-verify' if path.name.endswith('-logic-verify.jsonl') else 'verify'
    expected = {row['case']: row for row in reference[suffix]}
    actual = {row['case']: row for row in lines(path) if row['case'] != 'hipster_unrecorded'}
    assert actual == expected, path
    audit['traces'][str(path.relative_to(REPORT))] = len(expected)
for path in sorted((REPORT / 'validation').rglob('*-fuzz.jsonl')):
    assert lines(path) == reference['fuzz'], path
    audit['traces'][str(path.relative_to(REPORT))] = 56

raw = lines(REPORT / 'results/final-timings.jsonl')
header = raw[0]
assert header['build'] == meta
assert header['seed'] == 20261114 and header['cpu'] == 0
assert header['rounds'] == 21 and header['target_ms'] == 150
assert header['transition_signature_mode'] == 'on'
samples = [row for row in raw if row['kind'] == 'sample']
calibration = {row['case']: row for row in raw if row['kind'] == 'calibration'}
assert len(calibration) == 17 and len(samples) == 2142
groups = collections.defaultdict(list)
for row in samples:
    assert row['iterations'] == calibration[row['case']]['iterations']
    assert len(row['samples_cpu_ns']) == len(row['samples_ns']) == 1
    groups[row['case'], row['variant']].append(row['round'])
assert len(groups) == 102
assert all(sorted(rounds) == list(range(21)) for rounds in groups.values())
expected_hashes = {
    'master': meta['master']['variants']['reference']['sha256'],
    'previous': meta['signatures']['variants']['workload_previous']['sha256'],
    'selected': meta['signatures']['variants']['workload_signatures']['sha256'],
}
for name in ['master', 'previous', 'selected']:
    expected_hashes[name + '_pgo'] = meta['pgo-' + name]['builds']['pgo']['sha256']
    assert meta['pgo-' + name]['training'] == meta['pgo-master']['training']
    assert meta['pgo-' + name]['harness_sha256'] == meta['pgo-master']['harness_sha256']
assert {name: entry['sha256'] for name, entry in header['variants'].items()} == expected_hashes
audit['native'] = {'samples': len(samples), 'cases': len(calibration), 'variants': 6,
                   'rounds': 21, 'pairing_iterations_and_hashes_match': True}
cold = lines(REPORT / 'validation/cold/samples.jsonl')
assert len(cold) == 63
assert {row['case'] for row in cold} == {'first_signature_classification', 'warmed_signature_classification', 'reference_classification'}
for case in {row['case'] for row in cold}:
    rows = [row for row in cold if row['case'] == case]
    assert sorted(row['sample'] for row in rows) == list(range(21))
    assert all(row['table_bytes'] == 524288 and row['warmups'] == 3 for row in rows)
audit['isolated_classifier_measurements'] = len(cold)
(REPORT / 'validation/final-artifact-audit.json').write_text(json.dumps(audit, indent=2) + '\n')
checksum_path = REPORT / 'SHA256SUMS.json'
if checksum_path.exists():
    expected = read(checksum_path)
    actual = {str(path.relative_to(REPORT)): sha(path)
              for path in sorted(REPORT.rglob('*'))
              if path.is_file() and path != checksum_path}
    assert actual == expected, 'report artifact checksum mismatch'
print(f"Verified 511 tests, {len(audit['traces'])} trace files, 2142 native samples and source/binary hashes.")
