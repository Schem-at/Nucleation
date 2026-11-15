#!/usr/bin/env python3
"""Download pinned dynamic candidates and inspect their saved NBT, without Rust.

These are static candidates, not passing simulator or vanilla benchmarks.
Third-party binaries and generated inspection output remain under ignored tests/.
"""
import argparse
import collections
import gzip
import hashlib
import io
import json
import pathlib
import struct
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'docs/performance/schematic-candidates/dynamic-sources.json'


def read_nbt(data):
    stream = io.BytesIO(gzip.decompress(data))

    def number(fmt):
        return struct.unpack('>' + fmt, stream.read(struct.calcsize('>' + fmt)))[0]

    def string():
        return stream.read(number('H')).decode('utf-8')

    def payload(tag):
        if tag in range(1, 7):
            return number({1: 'b', 2: 'h', 3: 'i', 4: 'q', 5: 'f', 6: 'd'}[tag])
        if tag == 8:
            return string()
        if tag == 9:
            child, count = number('B'), number('i')
            return [payload(child) for _ in range(count)]
        if tag == 10:
            value = {}
            while child := number('B'):
                name = string()
                value[name] = payload(child)
            return value
        if tag in (7, 11, 12):
            return [number({7: 'b', 11: 'i', 12: 'q'}[tag]) for _ in range(number('i'))]
        raise ValueError(f'Unsupported NBT tag {tag}')

    tag = number('B')
    string()
    root = payload(tag)
    assert stream.read() == b'', 'Unexpected trailing NBT data'
    return root


def inspect(data):
    root = read_nbt(data)
    regions = []
    for name, region in root['Regions'].items():
        size = [abs(region['Size'][axis]) for axis in 'xyz']
        base = [region['Position'][axis] + min(0, region['Size'][axis] + 1) for axis in 'xyz']
        palette = region['BlockStatePalette']
        bits = max(2, (len(palette) - 1).bit_length())
        words = [word & ((1 << 64) - 1) for word in region['BlockStates']]
        materials = collections.Counter()
        pistons, controls = [], []
        sx, sy, sz = size
        for index in range(sx * sy * sz):
            word, offset = divmod(index * bits, 64)
            palette_index = words[word] >> offset
            if offset + bits > 64:
                palette_index |= words[word + 1] << (64 - offset)
            state = palette[palette_index & ((1 << bits) - 1)]
            block = state['Name']
            if block == 'minecraft:air':
                continue
            materials[block] += 1
            pos = [base[0] + index % sx, base[1] + index // (sx * sz),
                   base[2] + (index // sx) % sz]
            row = {'pos': pos, 'state': state}
            if block in ('minecraft:piston', 'minecraft:sticky_piston',
                         'minecraft:piston_head', 'minecraft:moving_piston'):
                pistons.append(row)
            if block in ('minecraft:lever', 'minecraft:note_block', 'minecraft:dispenser',
                         'minecraft:dropper', 'minecraft:hopper', 'minecraft:white_shulker_box'):
                controls.append(row)
        block_entities = region.get('TileEntities', [])
        regions.append({'name': name, 'saved_position': region['Position'],
                        'saved_size': region['Size'], 'minimum': base,
                        'dimensions': size, 'non_air_blocks': sum(materials.values()),
                        'materials': dict(sorted(materials.items())),
                        'initial_piston_states': pistons, 'potential_controls': controls,
                        'block_entity_count': len(block_entities),
                        'block_entity_types': dict(collections.Counter(be['id'] for be in block_entities)),
                        'block_entities': block_entities, 'entities': region.get('Entities', [])})
    total = sum(region['non_air_blocks'] for region in regions)
    assert total == root['Metadata']['TotalBlocks'], 'Decoded block count differs from saved metadata'
    return {'status': 'static-only', 'native_data_version': root['MinecraftDataVersion'],
            'metadata': root['Metadata'], 'regions': regions}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=pathlib.Path,
                        default=ROOT / 'tests/benchmark-fixtures/candidates/dynamic')
    args = parser.parse_args()
    manifest = json.loads(MANIFEST.read_text())
    args.output.mkdir(parents=True, exist_ok=True)
    results = {}
    for source in manifest['candidates']:
        target = args.output / (source['id'] + '.litematic')
        if target.exists():
            data = target.read_bytes()
        else:
            request = urllib.request.Request(source['download_url'], headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(request, timeout=30) as response:
                data = response.read(4 * 1024 * 1024 + 1)
        assert len(data) == source['bytes'], f"{source['id']}: byte count changed"
        assert hashlib.sha256(data).hexdigest() == source['sha256'], f"{source['id']}: hash changed"
        inspection = inspect(data)
        assert inspection['native_data_version'] == source['native_data_version']
        if not target.exists():
            target.write_bytes(data)
        results[source['id']] = {'source': source, 'inspection': inspection}
    output = args.output / 'inspection.json'
    output.write_text(json.dumps(results, indent=2) + '\n')
    print(f'Inspected {len(results)} static-only candidates: {output}')


if __name__ == '__main__':
    main()
