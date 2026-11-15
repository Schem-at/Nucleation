#!/usr/bin/env python3
"""Prepare native thumbnail jobs and verify/cache their results; never builds Rust.

Run the native renderer on harrison-work. Copy PNGs and result JSONL files back
before reconciling. Install blake3 and Pillow in an archive-local Python venv.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path
import zipfile


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
    temporary.replace(path)


def make_pack(args):
    destination = args.archive / 'rendered-thumbnails/assets'
    destination.mkdir(parents=True, exist_ok=True)
    output = destination / f'minecraft-{args.version}-models.zip'
    if output.exists():
        raise ValueError(f'Pack already exists: {output}; retain existing provenance or select a new version')
    with zipfile.ZipFile(args.client_jar) as source:
        bad = source.testzip()
        if bad: raise ValueError(f'Client JAR CRC failure: {bad}')
        names = sorted(n for n in source.namelist()
                       if n.startswith(('assets/minecraft/blockstates/', 'assets/minecraft/models/', 'assets/minecraft/textures/'))
                       and n.endswith(('.json', '.png', '.mcmeta')))
        if not names: raise ValueError('No Minecraft models/textures in client JAR')
        with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as pack:
            for name in names:
                info = zipfile.ZipInfo(name, (2020, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                pack.writestr(info, source.read(name))
    metadata = {'version': args.version, 'client_jar': str(args.client_jar),
                'client_sha256': sha256(args.client_jar), 'pack_sha256': sha256(output),
                'file': output.name, 'files': len(names), 'bytes': output.stat().st_size,
                'scope': 'Visual assets only; not a simulation/reference version.'}
    write_json(output.with_suffix('.json'), metadata)
    print(json.dumps(metadata))


def jobs(args):
    library = json.loads((args.archive / 'library.json').read_text())
    entries = {r['sha256']: r for r in reversed(library['entries']) if r['kind'] == 'schematic'}
    previous_path = args.archive / 'rendered-thumbnails/manifest.json'
    previous = json.loads(previous_path.read_text()).get('entries', {}) if previous_path.exists() else {}
    groups = collections.defaultdict(list)
    omitted = collections.Counter()
    for key, row in sorted(entries.items()):
        if args.source and row['source'] not in args.source: continue
        suffix = Path(row['path']).suffix.lower()
        if args.extension and suffix not in args.extension: continue
        if suffix == '.mcstructure': omitted['bedrock'] += 1; continue
        if row['source'] == 'user-world-record-plot' and not args.include_plot:
            omitted['whole_plot'] += 1; continue
        if args.resume and previous.get(key, {}).get('status') == 'ok':
            omitted['already_rendered'] += 1; continue
        local = (args.archive / row['local_path']).resolve()
        if not local.is_relative_to(args.archive.resolve()) or sha256(local) != key:
            raise ValueError(f'Archive input hash/path mismatch: {row["local_path"]}')
        version = '26.3' if (row.get('data_version') or 0) > 4671 else '1.21.11'
        groups[version].append({'id': key, 'input': str(args.input_root / row['local_path']),
                                'output': str(args.output_root / f'{key}.png'),
                                'local_path': row['local_path'], 'data_version': row.get('data_version')})
    args.jobs_dir.mkdir(parents=True, exist_ok=True)
    for version, group in sorted(groups.items()):
        path = args.jobs_dir / f'jobs-{version}.jsonl'
        path.write_text(''.join(json.dumps(r) + '\n' for r in group))
        (args.jobs_dir / f'files-{version}.txt').write_text(''.join(r['local_path'] + '\n' for r in group))
    print(json.dumps({'jobs': {k: len(v) for k, v in groups.items()}, 'omitted': omitted}))


def reconcile(args):
    from blake3 import blake3
    from PIL import Image
    root = args.archive / 'rendered-thumbnails'
    manifest_path = root / 'manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {'schema_version': 1, 'entries': {}}
    originals = {r['sha256']: args.archive / r['local_path'] for r in json.loads((args.archive / 'library.json').read_text())['entries']}
    successful = failed = 0
    for result_file in args.results:
        for line in result_file.read_text().splitlines():
            if not line.strip(): continue
            record = json.loads(line)
            key = record.get('id')
            if key not in originals: raise ValueError(f'Unknown result id {key}')
            source_bytes = originals[key].read_bytes()
            if hashlib.sha256(source_bytes).hexdigest() != key:
                raise ValueError(f'Original changed: {key}')
            if record['status'] == 'ok':
                if blake3(source_bytes).hexdigest() != record['input_blake3']:
                    raise ValueError(f'Rendered input differs from archive: {key}')
                path = root / 'pngs' / f'{key}.png'
                raw = path.read_bytes()
                if blake3(raw).hexdigest() != record['png_blake3']:
                    raise ValueError(f'PNG transfer/hash mismatch: {key}')
                with Image.open(path) as image:
                    image.verify()
                with Image.open(path) as image:
                    image.load()
                    if image.format != 'PNG' or image.size != (record['width'], record['height']):
                        raise ValueError(f'Invalid PNG dimensions: {key}')
                record['png_sha256'] = hashlib.sha256(raw).hexdigest()
                successful += 1
            else: failed += 1
            # All renderer warnings/provenance remain available in the API.
            manifest['entries'][key] = {k: v for k, v in record.items() if k not in ['input', 'output', 'id', 'job_line']}
    manifest['counts'] = dict(collections.Counter(r['status'] for r in manifest['entries'].values()))
    manifest['scope'] = 'Textured saved-state renders; zero simulated ticks. Rendering does not establish functionality or vanilla parity.'
    write_json(manifest_path, manifest)
    print(json.dumps({'verified_results': successful, 'error_results': failed, 'cache': manifest['counts']}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    commands = parser.add_subparsers(dest='command', required=True)
    pack = commands.add_parser('pack'); pack.add_argument('--client-jar', type=Path, required=True); pack.add_argument('--version', required=True)
    job = commands.add_parser('jobs')
    job.add_argument('--source', action='append'); job.add_argument('--resume', action='store_true'); job.add_argument('--include-plot', action='store_true')
    job.add_argument('--extension', action='append', help='Restrict to a saved suffix, e.g. .schematic; repeatable')
    job.add_argument('--input-root', type=Path, required=True); job.add_argument('--output-root', type=Path, required=True); job.add_argument('--jobs-dir', type=Path, required=True)
    result = commands.add_parser('reconcile'); result.add_argument('results', type=Path, nargs='+')
    args = parser.parse_args()
    {'pack': make_pack, 'jobs': jobs, 'reconcile': reconcile}[args.command](args)


if __name__ == '__main__': main()
