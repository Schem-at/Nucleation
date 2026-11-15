#!/usr/bin/env python3
"""Download pinned public schematic trees into a resumable local research archive.

Input: ARCHIVE/sources/NAME/{source,tree}.json. source has repository + commit;
tree is GitHub's complete recursive Git tree for that pinned commit. Preserve
these metadata files when moving the archive. Downloads never execute content.
"""
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import threading
import time
import urllib.parse
import urllib.request

SCHEMATICS = {'.schem', '.schematic', '.litematic', '.nbt', '.snbt', '.mcstructure'}
METADATA = {'.json', '.md', '.txt', '.yaml', '.yml', '.toml', '.mcfunction', '.tex', '.rsl', '.v', '.fixture'}
NAMES = {'license', 'copying', 'notice', 'readme'}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def atomic_json(path, value):
    tmp = path.with_suffix(path.suffix + '.tmp')
    tmp.write_text(json.dumps(value, indent=2) + '\n')
    tmp.replace(path)


def selected(path):
    p = PurePosixPath(path)
    return p.suffix.lower() in SCHEMATICS | METADATA or p.name.lower() in NAMES


def fetch(url, limit):
    for attempt in range(3):
        try:
            request = urllib.request.Request(url, headers={'User-Agent': 'Nucleation-schematic-research/1.0'})
            with urllib.request.urlopen(request, timeout=30) as response:
                data = response.read(limit + 1)
            if len(data) > limit:
                raise ValueError('download exceeds size limit')
            return data
        except Exception:
            if attempt == 2:
                raise
            time.sleep(1 + attempt)


def archive_source(root, directory, args):
    source = json.loads((directory / 'source.json').read_text())
    tree = json.loads((directory / 'tree.json').read_text())
    if tree.get('truncated'):
        raise ValueError(f'{directory}: truncated source tree')
    repository, commit = source['repository'], source['commit']
    if len(repository.split('/')) != 2 or len(commit) != 40 or not all(c in '0123456789abcdef' for c in commit):
        raise ValueError('Expected owner/repository and full pinned commit')
    records = []
    journal = directory / 'downloads.jsonl'
    prior = {}
    if journal.exists():
        saved = journal.read_bytes()
        chunks = saved.splitlines(keepends=True)
        for index, line in enumerate(chunks):
            try:
                row = json.loads(line)
            except (ValueError, UnicodeDecodeError):
                if index != len(chunks) - 1 or line.endswith(b'\n'):
                    raise
                journal.with_name(f'downloads-interrupted-{time.time_ns()}.jsonl').write_bytes(saved)
                journal.write_bytes(b''.join(chunks[:index]))
                break
            if row.get('status') == 'downloaded':
                prior[row['path']] = row
    lock = threading.Lock()
    output = directory / 'files'
    output.mkdir(exist_ok=True)
    objects = root / 'objects' / 'sha256'
    objects.mkdir(parents=True, exist_ok=True)
    prefixes = source.get('path_prefixes', [])
    entries = [row for row in tree['tree'] if row['type'] == 'blob' and selected(row['path'])
               and (not prefixes or any(row['path'].startswith(p) for p in prefixes)
                    or '/' not in row['path'])]

    def download(entry):
        relative = PurePosixPath(entry['path'])
        row = {'path': entry['path'], 'git_blob_sha1': entry['sha'],
               'source_bytes': entry.get('size'),
               'kind': ('schematic' if relative.suffix.lower() in SCHEMATICS else
                        'layout' if relative.suffix.lower() in source.get('layout_extensions', []) else 'metadata'),
               'validation': 'not_simulated'}
        try:
            if relative.is_absolute() or '..' in relative.parts:
                raise ValueError('unsafe source path')
            if entry.get('mode') not in ('100644', '100755'):
                raise ValueError('non-regular source file')
            if entry.get('size', 0) > args.max_file_mib * 1024**2:
                raise ValueError('source exceeds per-file limit')
            target = output.joinpath(*relative.parts)
            old = prior.get(entry['path'])
            if old and old['git_blob_sha1'] == entry['sha'] and target.is_file() and digest(target.read_bytes()) == old['sha256']:
                return old
            reserve = args.reserve_gib * 1024**3 + args.max_file_mib * 1024**2
            if shutil.disk_usage(root).free < reserve:
                raise ValueError('disk reserve reached; no download attempted')
            quoted = urllib.parse.quote(entry['path'], safe='/')
            url = f'https://raw.githubusercontent.com/{repository}/{commit}/{quoted}'
            row['url'] = url
            data = fetch(url, args.max_file_mib * 1024**2)
            git_hash = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            if git_hash != entry['sha']:
                raise ValueError('Git blob hash mismatch')
            if data.startswith(b'version https://git-lfs.github.com/spec/v1\n'):
                fields = dict(line.split(' ', 1) for line in data.decode().strip().splitlines())
                size = int(fields['size'])
                expected = fields['oid'].removeprefix('sha256:')
                if size > args.max_file_mib * 1024**2:
                    raise ValueError('LFS artifact exceeds per-file limit')
                row['lfs_url'] = f'https://media.githubusercontent.com/media/{repository}/{commit}/{quoted}'
                data = fetch(row['lfs_url'], size)
                if len(data) != size or digest(data) != expected:
                    raise ValueError('LFS content hash/size mismatch')
            sha = digest(data)
            blob = objects / sha[:2] / sha
            with lock:
                blob.parent.mkdir(exist_ok=True)
                if blob.exists():
                    if digest(blob.read_bytes()) != sha:
                        blob.rename(blob.with_name(f'{sha}.corrupt-{time.time_ns()}'))
                if not blob.exists():
                    if shutil.disk_usage(root).free < args.reserve_gib * 1024**3 + len(data):
                        raise ValueError('disk reserve reached before object write')
                    temporary = blob.with_name(f'{sha}.tmp-{time.time_ns()}')
                    try:
                        temporary.write_bytes(data)
                        temporary.replace(blob)
                    finally:
                        temporary.unlink(missing_ok=True)
                blob.chmod(0o444)
                target.parent.mkdir(parents=True, exist_ok=True)
                if target.exists():
                    if digest(target.read_bytes()) != sha:
                        raise ValueError('existing file differs; preserved without overwrite')
                else:
                    os.link(blob, target)
            row.update(status='downloaded', bytes=len(data), sha256=sha,
                       local_path=str(target.relative_to(root)), object_path=str(blob.relative_to(root)))
        except Exception as error:
            row.update(status='error', error=str(error))
        with lock:
            with journal.open('a') as file:
                file.write(json.dumps(row) + '\n')
        return row

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        for row in pool.map(download, entries):
            records.append(row)
            if len(records) % 100 == 0:
                print(f'{directory.name}: {len(records)}/{len(entries)} files', flush=True)
    manifest = {'source': source, 'tree_sha': tree['sha'], 'files': records,
                'selected_files': len(entries), 'downloaded': sum(r['status'] == 'downloaded' for r in records),
                'schematics': sum(r['status'] == 'downloaded' and r['kind'] == 'schematic' for r in records),
                'layouts': sum(r['status'] == 'downloaded' and r['kind'] == 'layout' for r in records),
                'errors': sum(r['status'] == 'error' for r in records),
                'downloaded_bytes': sum(r.get('bytes', 0) for r in records),
                'excluded_files': sum(r['type'] == 'blob' for r in tree['tree']) - len(entries)}
    atomic_json(directory / 'manifest.json', manifest)
    print(json.dumps({k: v for k, v in manifest.items() if k not in ['source', 'files']}), flush=True)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--source', action='append', default=[])
    parser.add_argument('--workers', type=int, default=6)
    parser.add_argument('--max-file-mib', type=int, default=64)
    parser.add_argument('--reserve-gib', type=int, default=5)
    args = parser.parse_args()
    root = args.archive.resolve()
    directories = sorted((root / 'sources').iterdir())
    for directory in directories:
        if (directory.is_dir() and (directory / 'tree.json').exists()
                and (not args.source or directory.name in args.source)):
            archive_source(root, directory, args)
    manifests = [json.loads(p.read_text()) for p in sorted((root / 'sources').glob('*/manifest.json'))]
    index = {'sources': [{k: v for k, v in m.items() if k != 'files'} for m in manifests],
             'schematic_files': sum(m['schematics'] for m in manifests),
             'layout_files': sum(m.get('layouts', 0) for m in manifests),
             'unique_schematic_contents': len({r['sha256'] for m in manifests for r in m['files'] if r['status'] == 'downloaded' and r['kind'] == 'schematic'}),
             'downloaded_files': sum(m['downloaded'] for m in manifests),
             'errors': sum(m['errors'] for m in manifests),
             'validation': 'Downloaded and hash-verified only; no simulation/parity claim.'}
    atomic_json(root / 'index.json', index)
    print(json.dumps({k: v for k, v in index.items() if k != 'sources'}), flush=True)


if __name__ == '__main__':
    main()
