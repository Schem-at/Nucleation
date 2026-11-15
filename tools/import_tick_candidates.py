#!/usr/bin/env python3
"""Import the eight inspected community candidates with the production Rust path."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--exporter', type=Path, required=True)
    p.add_argument('--sources', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    logic = json.loads((ROOT / 'docs/performance/schematic-candidates/logic-sources.json').read_text())['files']
    dynamic = json.loads((ROOT / 'docs/performance/schematic-candidates/dynamic-sources.json').read_text())['candidates']
    records = []
    for entry in logic + dynamic:
        logical = entry in logic
        source = a.sources / ('logic-archive' if logical else 'dynamic') / (entry['filename'] if logical else entry['id'] + '.litematic')
        assert sha(source) == entry['sha256'], source
        fixture = a.output / (entry['id'] + '.json')
        snbt = a.output / (entry['id'] + '.snbt')
        command = [str(a.exporter.resolve()), str(source.resolve()), str(fixture.resolve()), str(snbt.resolve())]
        run = subprocess.run(command, text=True, capture_output=True)
        (a.output / (entry['id'] + '.log')).write_text(run.stdout + run.stderr)
        record = {'id': entry['id'], 'source_sha256': sha(source), 'command': command, 'exit_code': run.returncode}
        if run.returncode == 0:
            data = json.loads(fixture.read_text())
            expected_version = entry['inspection']['data_version'] if logical else entry['native_data_version']
            expected_blocks = entry['inspection']['non_air_blocks'] if logical else entry['non_air_blocks']
            expected_size = entry['inspection']['dimensions'] if logical else entry['dimensions']
            assert data['data_version'] == expected_version, (entry['id'], 'save version changed')
            assert len(data['fixture']) == expected_blocks, (entry['id'], 'block count changed')
            assert data['size'] == expected_size, (entry['id'], 'dimensions changed')
            assert data['entity_count'] == 0
            record.update(status='imported', blocks=expected_blocks, data_version=expected_version,
                          size=data['size'], block_entities=data['block_entity_count'],
                          fixture_sha256=sha(fixture), snbt_sha256=sha(snbt))
        else:
            record['status'] = 'import_failed'
        records.append(record)
        (a.output / 'manifest.json').write_text(json.dumps({'exporter_sha256': sha(a.exporter),
            'exporter_source_sha256': sha(ROOT / 'examples/export_tick_oracle_fixture.rs'), 'imports': records}, indent=2) + '\n')
        print(json.dumps(record), flush=True)
    if any(r['status'] != 'imported' for r in records):
        raise SystemExit(1)


if __name__ == '__main__':
    main()
