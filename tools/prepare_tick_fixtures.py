#!/usr/bin/env python3
"""Prepare attributed external benchmarks with pinned input hashes.

Build export_tick_fixture on the benchmark host first. This script downloads
community files, verifies every input, and uses the production importer/exporter.
Downloaded and generated structures stay outside Git; provenance stays here.
"""
import argparse, hashlib, json, pathlib, subprocess, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
LITHIUM = ROOT / 'tests/corpus/lithium-litematic/gametest/structure'
SOURCES = {
    'item_sorter': ('153dfbd644afd9188bbeb126cf71c9f2a61bb22e9400c56bdca8ca45315b7509', None),
    'hopper_transfer_speed': ('ecedb38e736d63df79b368bae3bf91d4026dd143550e8b52550f7177c0055517', None),
    'hopper_storagecart_interaction': ('044f9e91485043534e456c3a0eb71c745390fc0547457aa2ea514dc37ba565b2', None),
    'comparator_update_collection': ('b8c125d18415481a58f2a48a971749554d16b579edf9f6a9040cf5632ec7a0b0', None),
    'shutter4': ('b4262fe85e36a8ca6104eb864c5f58e60725e5be0b08de2d8152c32294cecd74',
        'https://drive.google.com/uc?export=download&id=1LEfyHchgc7Wwax9cfh_1S0fozOQvt-6G'),
    'trapdoor2x3': ('db6d33e785c445eaf633ddca50dc965aba8f73c3b0c221e6239238810a11f6a9',
        'https://archive.org/download/2x-3-flush-piston-trapdoor/2x3%20flush%20piston%20trapdoor.litematic'),
}
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--exporter', type=pathlib.Path, required=True)
p.add_argument('--output', type=pathlib.Path, default=ROOT / 'tests/benchmark-fixtures')
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=True)
(a.output / 'sources').mkdir(exist_ok=True)
manifest = {'exporter_sha256': hashlib.sha256(a.exporter.read_bytes()).hexdigest(), 'fixtures': {}}
for name, (expected, url) in SOURCES.items():
    source = (a.output / 'sources' if url else LITHIUM) / (name + '.litematic')
    if not source.exists():
        assert url, f'Missing repository fixture: {source}'
        with urllib.request.urlopen(url, timeout=60) as response:
            data = response.read(16 * 1024 * 1024 + 1)
        assert len(data) <= 16 * 1024 * 1024, 'Unexpectedly large download'
        assert hashlib.sha256(data).hexdigest() == expected, f'{name}: download changed; inspect before updating pin'
        source.write_bytes(data)
    assert hashlib.sha256(source.read_bytes()).hexdigest() == expected, f'{source}: input hash mismatch'
    target = a.output / (name + '.snbt')
    spec = a.output / (name + '.test.json')
    subprocess.run([str(a.exporter.resolve()), str(source), str(target), str(spec)], check=True)
    manifest['fixtures'][name] = {'input_sha256': expected, 'url': url,
        'source': str(source), 'snbt_sha256': hashlib.sha256(target.read_bytes()).hexdigest(),
        'spec_sha256': hashlib.sha256(spec.read_bytes()).hexdigest()}
(a.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(f'Prepared {len(SOURCES)} external fixtures in {a.output}')
