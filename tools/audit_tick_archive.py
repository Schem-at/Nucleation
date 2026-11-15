#!/usr/bin/env python3
"""Audit local schematic archive bytes and save compact recovery metadata.

The normal audit performs no network access and does not simulate schematics.
It saves source configurations/pinned tree URLs, an integrity report, and
deterministic gzip copies of the curation JSONs. --refresh-index updates only
the downloader's aggregate index.json after a successful audit.

Recovery (choose a new ARCHIVE directory):
  --restore-sources SNAPSHOT restores source.json configurations, without fetching.
  Fetch each snapshot source's tree.url into sources/ID/tree.json, then run
  tools/archive_tick_schematics.py --archive ARCHIVE (preserve the disk reserve).
  --restore-curation SNAPSHOT expands the verified tracked catalog/*.json.gz.
  For manual candidates, run tools/prepare_tick_dynamic_candidates.py --output
  ARCHIVE/sources/curated-candidates/files. For local_only sources, explicitly
  copy the preserved local original into the manifest's local_path under ARCHIVE.
  Then --register-manual SNAPSHOT verifies all supplied bytes and restores
  objects/manifests. Local-only files are never fetched from an external service.
  Finally rerun this audit. None of the recovery modes fetches or executes code.
"""
import argparse
import collections
import datetime
import gzip
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import tempfile


REPO = Path(__file__).resolve().parents[1]
DOCS = REPO / 'docs/performance/schematic-candidates'


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def git_blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def atomic_write(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix=path.name + '.', delete=False) as file:
        temporary = Path(file.name)
        try:
            file.write(data)
            file.close()
            temporary.replace(path)
        finally:
            temporary.unlink(missing_ok=True)


def safe_path(root, relative):
    if not isinstance(relative, str) or not relative:
        raise ValueError('empty/non-string relative path')
    path = PurePosixPath(relative)
    if path.is_absolute() or '..' in path.parts or '\\' in relative:
        raise ValueError(f'unsafe relative path: {relative!r}')
    target = root.joinpath(*path.parts)
    if not target.resolve().is_relative_to(root.resolve()):
        raise ValueError(f'path escapes archive: {relative!r}')
    return target


def source_id(value):
    if not isinstance(value, str) or not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', value):
        raise ValueError(f'invalid source id: {value!r}')
    return value


def preserve_write(path, data):
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError(f'existing different file preserved: {path}')
    else:
        atomic_write(path, data)


def deterministic_gzip(data):
    buffer = io.BytesIO()
    with gzip.GzipFile(filename='', mode='wb', fileobj=buffer, compresslevel=9, mtime=0) as file:
        file.write(data)
    return buffer.getvalue()


def restore(root, snapshot_path, mode):
    snapshot = json.loads(snapshot_path.read_text())
    if snapshot.get('schema_version') != 1:
        raise ValueError('unsupported recovery snapshot schema')
    if mode == 'sources':
        for item in snapshot['sources']:
            config = item['config']
            directory = root / 'sources' / source_id(config['id'])
            target = directory / 'source.json'
            if target.exists() and json.loads(target.read_text()) != config:
                raise ValueError(f'existing different source config preserved: {target}')
            if not target.exists():
                atomic_write(target, json_bytes(config))
            if 'manual_manifest' in item:
                preserve_write(directory / 'manual-downloads.json', json_bytes(item['manual_manifest']['files']))
            instruction = ('explicit local copy required; original URI in snapshot'
                           if config.get('local_only') else 'manual per-file URLs in snapshot')
            print(f"{config['id']}: " + (item.get('tree', {}).get('url') or instruction))
    elif mode == 'curation':
        for item in snapshot['curation_archives']:
            compressed = safe_path(snapshot_path.parent, item['snapshot_path']).read_bytes()
            if sha256(compressed) != item['sha256']:
                raise ValueError(f"compressed catalog hash mismatch: {item['snapshot_path']}")
            data = gzip.decompress(compressed)
            if sha256(data) != item['uncompressed_sha256'] or len(data) != item['uncompressed_bytes']:
                raise ValueError(f"expanded catalog hash/size mismatch: {item['snapshot_path']}")
            preserve_write(safe_path(root, item['archive_path']), data)
    elif mode == 'manual':
        for item in snapshot['sources']:
            if 'manual_manifest' not in item:
                continue
            manifest = item['manual_manifest']
            sid = source_id(item['config']['id'])
            for record in manifest['files']:
                local = safe_path(root, record['local_path'])
                data = local.read_bytes()
                if sha256(data) != record['sha256'] or len(data) != record['bytes']:
                    raise ValueError(f'manual content hash/size mismatch: {local}')
                preserve_write(safe_path(root, record['object_path']), data)
            preserve_write(root / 'sources' / sid / 'manifest.json', json_bytes(manifest))
            if 'manual_provenance' in item:
                preserve_write(root / 'sources' / sid / 'candidate-provenance.json', json_bytes(item['manual_provenance']))


def audit(args):
    root = args.archive.resolve()
    spec = importlib.util.spec_from_file_location('archive_tick_selection', REPO / 'tools/archive_tick_schematics.py')
    downloader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(downloader)
    errors, warnings = [], []
    manifests, sources, source_reports, catalog_reports = [], [], [], []
    records_by_key = {}
    unique_contents, unique_by_kind = {}, collections.defaultdict(dict)
    totals = collections.Counter()
    manifests_paths = sorted((root / 'sources').glob('*/manifest.json'))
    if not manifests_paths:
        errors.append({'type': 'no_manifests', 'path': str(root / 'sources')})

    def error(kind, **details):
        errors.append({'type': kind, **details})

    for manifest_path in manifests_paths:
        manifest_data = manifest_path.read_bytes()
        manifest = json.loads(manifest_data)
        manifests.append(manifest)
        config_path = manifest_path.with_name('source.json')
        config_data = config_path.read_bytes()
        config = json.loads(config_data)
        sid = source_id(config['id'])
        if manifest_path.parent.name != sid or manifest['source'] != config:
            error('source_config_mismatch', source=sid)
        source_snapshot = {'config': config, 'source_json_sha256': sha256(config_data),
                           'manifest_sha256': sha256(manifest_data)}
        tree_path = manifest_path.with_name('tree.json')
        tree_entries = None
        if 'repository' in config:
            repository, commit = config['repository'], config['commit']
            if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository) or not re.fullmatch(r'[0-9a-f]{40}', commit):
                raise ValueError(f'invalid pinned repository/commit: {sid}')
            if not tree_path.exists():
                error('missing_tree', source=sid)
            else:
                tree_data = tree_path.read_bytes()
                tree = json.loads(tree_data)
                if tree.get('truncated'):
                    error('truncated_tree', source=sid)
                if manifest.get('tree_sha') != tree.get('sha'):
                    error('tree_sha_mismatch', source=sid)
                tree_entries = {row['path']: row for row in tree['tree'] if row['type'] == 'blob'}
                prefixes = config.get('path_prefixes', [])
                expected_paths = {path for path in tree_entries if downloader.selected(path)
                                  and (not prefixes or '/' not in path or any(path.startswith(prefix) for prefix in prefixes))}
                actual_paths = {row['path'] for row in manifest['files']}
                if expected_paths != actual_paths:
                    error('tree_selection_coverage_mismatch', source=sid,
                          missing=sorted(expected_paths - actual_paths), unexpected=sorted(actual_paths - expected_paths))
                source_snapshot['tree'] = {
                    'url': f'https://api.github.com/repos/{repository}/git/trees/{commit}?recursive=1',
                    'archive_path': str(tree_path.relative_to(root)), 'sha': tree.get('sha'),
                    'saved_json_sha256': sha256(tree_data), 'saved_json_bytes': len(tree_data),
                    'entries': len(tree['tree']), 'blobs': len(tree_entries), 'truncated': bool(tree.get('truncated'))}
        else:
            source_snapshot['manual_manifest'] = manifest
            source_snapshot['manual_recovery'] = {
                'mode': 'explicit-local-copy' if config.get('local_only') else 'per-file-download',
                'files': [{'path': row['path'], 'original_url': row.get('url', row.get('source_url', config.get('url'))),
                           'destination_under_archive': row['local_path'], 'sha256': row['sha256'], 'bytes': row['bytes']}
                          for row in manifest['files']],
                'note': ('Preserve the user original independently. Explicitly copy it into the recorded archive destination; no public URL or external retrieval is implied.'
                         if config.get('local_only') else 'Recover each exact content from its recorded source; verify the saved hash before registration.')}
            provenance = manifest_path.with_name('candidate-provenance.json')
            if provenance.exists():
                source_snapshot['manual_provenance'] = json.loads(provenance.read_text())
            warnings.append({'type': 'local_manual_source_no_git_tree' if config.get('local_only') else 'manual_source_no_git_tree', 'source': sid,
                             'meaning': ('Verified against per-file SHA-256 and user-local provenance. The original binary is not in the tracked recovery metadata.'
                                         if config.get('local_only') else 'Verified against per-file SHA-256 and saved provenance; URLs may be mutable or expiring.')})
        sources.append(source_snapshot)
        counts = collections.Counter()
        for row in manifest['files']:
            path = row['path']
            safe_path(root, path)
            key = (sid, path)
            if key in records_by_key:
                error('duplicate_manifest_key', source=sid, path=path)
            records_by_key[key] = row
            counts['manifest_records'] += 1
            if row.get('status') != 'downloaded':
                error('not_downloaded', source=sid, path=path, status=row.get('status'))
                continue
            counts['downloaded_files'] += 1
            kind = row['kind']
            counts[kind + '_files'] += 1
            counts['manifested_bytes'] += row.get('bytes', 0)
            counts[kind + '_bytes'] += row.get('bytes', 0)
            try:
                expected_local = f'sources/{sid}/files/{path}'
                if row['local_path'] != expected_local:
                    error('unexpected_local_path', source=sid, path=path, local_path=row['local_path'])
                data = safe_path(root, row['local_path']).read_bytes()
                if len(data) != row['bytes']:
                    error('size_mismatch', source=sid, path=path, actual=len(data), expected=row['bytes'])
                actual = sha256(data)
                if actual != row['sha256']:
                    error('sha256_mismatch', source=sid, path=path, actual=actual, expected=row['sha256'])
                else:
                    counts['sha256_verified'] += 1
                unique_contents[actual] = len(data)
                unique_by_kind[kind][actual] = len(data)
                expected_object = f"objects/sha256/{row['sha256'][:2]}/{row['sha256']}"
                if row.get('object_path') != expected_object:
                    error('unexpected_object_path', source=sid, path=path, object_path=row.get('object_path'))
                obj = safe_path(root, row['object_path']).read_bytes()
                if len(obj) != row['bytes'] or sha256(obj) != row['sha256']:
                    error('object_hash_or_size_mismatch', source=sid, path=path)
                else:
                    counts['object_verified'] += 1
                if 'git_blob_sha1' in row:
                    # LFS rows store artifact bytes but a pointer Git blob ID.
                    git_data = (f"version https://git-lfs.github.com/spec/v1\noid sha256:{actual}\nsize {len(data)}\n".encode()
                                if row.get('lfs_url') else data)
                    if git_blob(git_data) != row['git_blob_sha1']:
                        error('git_blob_mismatch', source=sid, path=path, lfs_pointer=bool(row.get('lfs_url')))
                    else:
                        counts['git_blob_verified'] += 1
                    if tree_entries is not None:
                        entry = tree_entries.get(path)
                        if not entry or entry['sha'] != row['git_blob_sha1'] or entry.get('size') != row.get('source_bytes'):
                            error('manifest_tree_entry_mismatch', source=sid, path=path)
                        else:
                            counts['tree_entry_verified'] += 1
                elif 'repository' in config:
                    error('repository_file_missing_git_hash', source=sid, path=path)
            except (OSError, ValueError, KeyError) as exc:
                error('file_check_error', source=sid, path=path, detail=str(exc))
        for field, measured in [('downloaded', counts['downloaded_files']), ('schematics', counts['schematic_files']),
                                ('layouts', counts['layout_files']), ('downloaded_bytes', counts['manifested_bytes'])]:
            if manifest.get(field, 0) != measured:
                error('manifest_aggregate_mismatch', source=sid, field=field, saved=manifest.get(field, 0), measured=measured)
        totals.update(counts)
        source_reports.append({'source': sid, 'manifest_sha256': sha256(manifest_data), **dict(counts)})

    for directory in sorted((root / 'sources').iterdir()):
        if directory.is_dir() and not (directory / 'manifest.json').exists():
            error('source_without_manifest', source=directory.name)

    curated_keys, shortlist_keys, negative_keys = set(), set(), set()
    readiness_counts = collections.Counter()
    curation_reports = []
    for path in sorted((root / 'curation').glob('*.json')):
        data = path.read_bytes()
        curation = json.loads(data)
        curator = curation.get('curator', path.stem)
        local_keys = set()
        for group in ['entries', 'shortlist']:
            for row in curation.get(group, []):
                key = (row.get('source'), row.get('path'))
                match = records_by_key.get(key)
                if match is None:
                    error('curation_orphan', curator=curator, group=group, source=key[0], path=key[1])
                elif row.get('sha256') and row['sha256'] != match.get('sha256'):
                    error('curation_hash_mismatch', curator=curator, source=key[0], path=key[1])
                if group == 'entries':
                    if key in local_keys:
                        error('duplicate_curation_key', curator=curator, source=key[0], path=key[1])
                    local_keys.add(key)
                    curated_keys.add(key)
                    readiness_counts[row.get('readiness', 'unspecified')] += 1
                    if row.get('readiness') == 'known-negative':
                        negative_keys.add(key)
                else:
                    shortlist_keys.add(key)
                    if key not in local_keys:
                        error('shortlist_without_curator_entry', curator=curator, source=key[0], path=key[1])
        compressed = deterministic_gzip(data)
        target = args.catalog_dir / (path.name + '.gz')
        atomic_write(target, compressed)
        catalog = {'curator': curator, 'archive_path': str(path.relative_to(root)),
                   'snapshot_path': os.path.relpath(target, args.sources_output.parent),
                   'sha256': sha256(compressed), 'bytes': len(compressed),
                   'uncompressed_sha256': sha256(data), 'uncompressed_bytes': len(data)}
        catalog_reports.append(catalog)
        curation_reports.append({'curator': curator, 'entries': len(curation.get('entries', [])),
                                 'shortlist': len(curation.get('shortlist', [])),
                                 'readiness_counts': dict(collections.Counter(row.get('readiness', 'unspecified') for row in curation.get('entries', []))),
                                 **catalog})

    summary = {'sources': len(sources), **dict(totals),
               'unique_contents': len(unique_contents), 'unique_content_bytes': sum(unique_contents.values()),
               'unique_schematic_contents': len(unique_by_kind['schematic']),
               'unique_layout_contents': len(unique_by_kind['layout']),
               'unique_schematic_bytes': sum(unique_by_kind['schematic'].values()),
               'unique_layout_bytes': sum(unique_by_kind['layout'].values()),
               'curators': len(curation_reports), 'curation_entries': sum(c['entries'] for c in curation_reports),
               'curated_unique_keys': len(curated_keys), 'shortlist_entries': sum(c['shortlist'] for c in curation_reports),
               'shortlist_unique_keys': len(shortlist_keys), 'known_negative_unique_keys': len(negative_keys),
               'curation_readiness_counts': dict(readiness_counts)}
    index_refreshed = False
    if args.refresh_index and not errors:
        index = {'sources': [{k: v for k, v in m.items() if k != 'files'} for m in manifests],
                 'schematic_files': totals['schematic_files'], 'layout_files': totals['layout_files'],
                 'unique_schematic_contents': summary['unique_schematic_contents'],
                 'downloaded_files': totals['downloaded_files'], 'errors': 0,
                 'validation': 'Downloaded and hash-verified only; no simulation/parity claim.'}
        atomic_write(root / 'index.json', json_bytes(index))
        index_refreshed = True
    report = {'schema_version': 1, 'audited_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'archive_root': str(root), 'status': 'passed' if not errors else 'failed',
              'scope': 'Local content integrity, saved-tree consistency and curation joins. No network revalidation, builds, simulation or new parity proof.',
              'byte_accounting': 'manifested_bytes counts every source/path occurrence; unique_content_bytes deduplicates SHA-256 contents and is not filesystem allocated space.',
              'summary': summary, 'sources': source_reports, 'curation': curation_reports,
              'errors': errors, 'warnings': warnings, 'index_refreshed': index_refreshed,
              'helper_sha256': sha256(Path(__file__).read_bytes())}
    snapshot = {'schema_version': 1, 'sources': sources, 'curation_archives': catalog_reports,
                'downloader': {'path': 'tools/archive_tick_schematics.py', 'sha256': sha256((REPO / 'tools/archive_tick_schematics.py').read_bytes())},
                'recovery': [
                    'Run audit_tick_archive.py --archive NEW_ROOT --restore-sources this-snapshot.json.',
                    'For every Git source, GET its exact tree.url into tree.archive_path below NEW_ROOT. Preserve the full recursive response; reject truncated=true. Tree JSON serialization can differ, so saved_json_sha256 describes the original local snapshot, not an HTTP-byte requirement.',
                    'Run archive_tick_schematics.py --archive NEW_ROOT --reserve-gib 5. It selects source-config path prefixes/extensions and verifies downloaded Git blobs.',
                    'For curated-candidates run prepare_tick_dynamic_candidates.py --output NEW_ROOT/sources/curated-candidates/files. It verifies each original per-file SHA-256; mutable/expired source URLs may need refreshing without changing expected content.',
                    'For every local_only source, explicitly copy the preserved original local file to manual_recovery.files[].destination_under_archive below NEW_ROOT. The snapshot retains its original file URI/hash, not its bytes; never infer or fetch an external replacement.',
                    'Run audit_tick_archive.py --archive NEW_ROOT --register-manual this-snapshot.json to verify and register all supplied manual files/objects/manifests, including local-only originals.',
                    'Run audit_tick_archive.py --archive NEW_ROOT --restore-curation this-snapshot.json to expand the adjacent verified catalog gzip files.',
                    'Run the full audit with --refresh-index. Rebuild library.json/index.html separately using the parent indexer; this tool does not modify those views.',
                    'This metadata snapshot does not back up third-party binary bytes. Keep the persistent local archive independently; remote retention is not guaranteed.'
                ]}
    atomic_write(args.sources_output, json_bytes(snapshot))
    atomic_write(args.output, json_bytes(report))
    print(json.dumps({'status': report['status'], 'summary': summary, 'errors': errors, 'index_refreshed': index_refreshed}, ensure_ascii=False))
    return 0 if not errors else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=DOCS / 'archive-audit.json')
    parser.add_argument('--sources-output', type=Path, default=DOCS / 'archive-sources.json')
    parser.add_argument('--catalog-dir', type=Path, default=DOCS / 'catalog')
    parser.add_argument('--refresh-index', action='store_true')
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--restore-sources', type=Path, metavar='SNAPSHOT')
    mode.add_argument('--restore-curation', type=Path, metavar='SNAPSHOT')
    mode.add_argument('--register-manual', type=Path, metavar='SNAPSHOT')
    args = parser.parse_args()
    for key, operation in [('restore_sources', 'sources'), ('restore_curation', 'curation'), ('register_manual', 'manual')]:
        if getattr(args, key):
            restore(args.archive.resolve(), getattr(args, key).resolve(), operation)
            return 0
    return audit(args)


if __name__ == '__main__':
    raise SystemExit(main())
