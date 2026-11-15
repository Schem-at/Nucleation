#!/usr/bin/env python3
"""Import a user-supplied schematic into the external Atlas archive without Git."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

SOURCE = 'user-local-schematics'


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_suffix(path.suffix + '.tmp')
    tmp.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
    tmp.replace(path)


def copy_exact(path, data):
    if path.exists() and path.read_bytes() != data:
        raise ValueError(f'Existing archive file has different bytes: {path}')
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists(): path.write_bytes(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('file', type=Path)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--title')
    parser.add_argument('--authors')
    args = parser.parse_args()
    root, original = args.archive.resolve(), args.file.resolve(strict=True)
    existing_parent = root
    while not existing_parent.exists():
        existing_parent = existing_parent.parent
    if subprocess.run(['git', '-C', str(existing_parent), 'rev-parse', '--show-toplevel'], capture_output=True).returncode == 0:
        raise ValueError('Keep schematic archives outside Git')
    data = original.read_bytes(); sha = hashlib.sha256(data).hexdigest()
    directory = root / 'sources' / SOURCE
    manifest_path = directory / 'manifest.json'
    manifest = json.loads(manifest_path.read_bytes()) if manifest_path.is_file() else {'files': []}
    source = {'id': SOURCE, 'url': 'file://local-user-schematics', 'category': 'user-provided schematics',
              'local_only': True, 'license': 'Source-specific; local copy only',
              'provenance': 'Explicitly supplied local originals. Schematic bytes are retained outside Git.'}
    name = original.name
    previous = next((r for r in manifest['files'] if r['path'] == name), None)
    if previous and previous['sha256'] != sha:
        name = f'{original.stem}-{sha[:12]}{original.suffix}'
    local = directory / 'files' / name
    blob = root / 'objects/sha256' / sha[:2] / sha
    copy_exact(blob, data); copy_exact(local, data)
    existing = next((r for r in manifest['files'] if r['path'] == name), {})
    row = {**existing, 'path': name, 'kind': 'schematic', 'status': 'downloaded', 'bytes': len(data), 'sha256': sha,
           'source_url': original.as_uri(), 'local_only': True,
           'local_path': local.relative_to(root).as_posix(), 'object_path': blob.relative_to(root).as_posix()}
    if args.title: row['title'] = args.title
    if args.authors: row['authors'] = args.authors
    manifest['files'] = [r for r in manifest['files'] if r['path'] != name] + [row]
    manifest.update(source=source, selected_files=len(manifest['files']), downloaded=len(manifest['files']),
                    schematics=len(manifest['files']), layouts=0, errors=0,
                    downloaded_bytes=sum(r['bytes'] for r in manifest['files']))
    save(directory/'source.json', source); save(manifest_path, manifest)
    manifests = [json.loads(p.read_bytes()) for p in sorted((root/'sources').glob('*/manifest.json'))]
    save(root/'index.json', {'sources': [{k:v for k,v in m.items() if k!='files'} for m in manifests],
         'schematic_files': sum(m['schematics'] for m in manifests), 'layout_files': sum(m.get('layouts',0) for m in manifests),
         'unique_schematic_contents': len({r['sha256'] for m in manifests for r in m['files'] if r['status']=='downloaded' and r['kind']=='schematic'}),
         'downloaded_files': sum(m['downloaded'] for m in manifests), 'errors': sum(m['errors'] for m in manifests),
         'validation': 'Downloaded and hash-verified only; no simulation/parity claim.'})
    print(json.dumps({'sha256': sha, 'source': SOURCE, 'local_path': row['local_path'],
                      'atlas_url': f'http://127.0.0.1:8767/#entry={sha}', 'original_unchanged': hashlib.sha256(original.read_bytes()).hexdigest()==sha}, indent=2))

if __name__ == '__main__': main()
