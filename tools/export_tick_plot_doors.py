#!/usr/bin/env python3
"""Collect existing source-preserving plot extractions by their recorded evidence.

No simulation is performed, no inventory is repaired and no original is modified.
Schematic bytes and full evidence must remain outside Git. The strict verified
set requires an independent functional oracle AND activated/idle vanilla parity.
"""
from __future__ import annotations
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import zipfile

REPO = Path(__file__).resolve().parents[1]
SOURCES = {'world-record-parts', 'world-record-floor-parts'}
FAILURES = {'functional_fail', 'vanilla_functional_fail', 'vanilla_tick_end_parity_fail', 'engine_unsupported'}
TIERS = {
    'verified': 'Verified open/close and vanilla tick-end parity',
    'repeatable': 'Observed alternating states and exact repeat; aperture not verified',
    'responsive': 'Observed piston response and deterministic replay; complete cycle not verified',
}
NEGATIVE_SIGN = re.compile(r"\b(broken now|doesn.t work|does not work|not working|no longer works)\b", re.I)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + '\n').encode()


def inside(root, relative):
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'Path escapes archive: {relative}')
    return path


def negative_signs(row):
    return [sign for sign in row.get('signs', [])
            if NEGATIVE_SIGN.search(' '.join(str(line) for lines in sign.get('text', {}).values() for line in lines))]


def usable_trials(raw):
    assessment = (raw.get('native') or {}).get('assessment', {})
    if raw.get('status') != 'assessed' or assessment.get('status') != 'assessed':
        return []
    trials = assessment.get('trials', [])
    if not trials or any(t.get('deterministic_after_restore_and_reseed') is not True for t in trials):
        raise ValueError('Assessed result does not have complete deterministic replay')
    return [t for t in trials if t.get('control') is not None
            and t['first'].get('status') == 'completed'
            and t['first'].get('completed_ticks') == t['first'].get('requested_ticks')
            and t['first'].get('piston_related_changes', 0) > 0
            and t['first'].get('changes_away_from_control', 0) > 0
            and t['first'].get('actions')
            and all(a.get('status') == 'used' for a in t['first']['actions'])]


def alternating(trial):
    endpoints = trial['first'].get('endpoints', [])
    if len(endpoints) < 5 or any(e.get('moving_blocks', 0) for e in endpoints[:5]):
        return False
    states = [e.get('observable_state_fingerprint') for e in endpoints[:5]]
    return (all(states) and states[0] == states[2] == states[4]
            and states[1] == states[3] and states[0] != states[1])


def verified(status, source_sha):
    oracle = status.get('plot_validation', {}).get('oracle', {})
    if not (status.get('validation_status') == 'vanilla_tick_end_parity_pass'
            and status.get('functional_status') == 'vanilla_functional_pass'
            and status.get('benchmark_status') == 'admitted'
            and oracle.get('source_sha256') == source_sha and oracle.get('promoted') is True
            and oracle.get('functional_status') == 'vanilla_functional_pass'
            and not oracle.get('functional_failures')):
        return False
    for mode in ('cycle', 'idle'):
        result = oracle.get('native', {}).get(mode, {})
        frames = result.get('expected_frames', 0)
        fingerprint = result.get('compared_capture_state_blake3')
        if not (result.get('status') == 'matches_capture' and frames > 1
                and result.get('matching_frames_including_initial') == frames
                and result.get('first_difference') is None and fingerprint
                and fingerprint == result.get('compared_engine_state_blake3')
                and (result.get('actions', 0) > 0 if mode == 'cycle' else result.get('actions') == 0)):
            return False
    return True


def write_identical(path, data):
    if path.exists():
        if path.read_bytes() != data:
            raise ValueError(f'Refusing to overwrite different export: {path}')
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def export(archive, native):
    archive = archive.resolve()
    # A binary export must never land inside this or another checkout.
    git = subprocess.run(['git', '-C', str(archive), 'rev-parse', '--show-toplevel'], capture_output=True)
    if git.returncode == 0:
        raise ValueError('Keep the schematic archive outside Git; only export tools belong in the repository')
    library_bytes = (archive / 'library.json').read_bytes()
    status_bytes = (archive / 'validation/status.json').read_bytes()
    index_bytes = (native / 'index.json').read_bytes()
    manifest_bytes = (native / 'manifest.json').read_bytes()
    library, statuses = json.loads(library_bytes), json.loads(status_bytes)
    index, manifest = json.loads(index_bytes), json.loads(manifest_bytes)
    config = manifest['run_config']
    if index['run_config'] != config:
        raise ValueError('Mismatched native evidence configuration')
    rows = [r for r in library['entries'] if r.get('source') in SOURCES
            and r.get('classification') == 'door_candidate'
            and 'infrastructure_review' not in r.get('review_tags', [])]
    expected = {r['sha256'] for r in rows}
    if expected != set(index['results']) or expected != {r['sha256'] for r in manifest['candidates']}:
        raise ValueError('Native assessment does not cover every current door candidate')
    adapters = {}
    for p in sorted((REPO / 'circuit-adapters').glob('*/adapter.json')):
        profile = json.loads(p.read_bytes()); adapters[profile['source_sha256']] = (p.parent, profile)
    provenance = {'library_sha256': digest(library_bytes), 'status_sha256': digest(status_bytes),
                  'native_index_sha256': digest(index_bytes), 'native_manifest_sha256': digest(manifest_bytes),
                  'exporter_sha256': digest(Path(__file__).read_bytes()), 'native_run_config': config}
    # All evidence and sources are validated before publishing the current pointer.
    selected, excluded, payloads = [], [], {}
    for row in sorted(rows, key=lambda r: (r['source'], r['path'])):
        key = row['sha256']; data = inside(archive, row['local_path']).read_bytes()
        if digest(data) != key:
            raise ValueError(f'Changed source bytes: {row["local_path"]}')
        result_path = inside(native, index['results'][key]['result'])
        raw_bytes = result_path.read_bytes(); raw = json.loads(raw_bytes)
        if raw['source_sha256'] != key or raw['run_config'] != config or raw['status'] != index['results'][key]['status']:
            raise ValueError(f'Mismatched evidence for {key}')
        status = statuses.get(key, {})
        native_overlay = status.get('plot_validation', {}).get('native', {})
        if native_overlay.get('binary_sha256') != config['binary_sha256']:
            raise ValueError(f'Stale status overlay for {key}')
        trials = usable_trials(raw)
        why = ('known_validation_failure' if status.get('validation_status') in FAILURES else
               'source_labelled_broken' if negative_signs(row) else
               raw['status'] if raw['status'] != 'assessed' else
               'no_repeatable_piston_control_response' if not trials else None)
        if why:
            excluded.append({'sha256': key, 'source': row['source'], 'path': row['path'], 'reason': why,
                             'validation_status': status.get('validation_status'), 'negative_signs': negative_signs(row)})
            continue
        tier = 'verified' if verified(status, key) else 'repeatable' if any(alternating(t) for t in trials) else 'responsive'
        if tier == 'verified':
            evidence = inside(archive, status['validation_evidence']).read_bytes()
            if digest(evidence) != status['plot_validation']['oracle']['report_sha256']:
                raise ValueError('Verified oracle report changed')
        trials.sort(key=lambda t: (not alternating(t), -t['first']['piston_related_changes'], t['control']))
        chosen = trials[0]
        adapter = adapters.get(key) if tier == 'verified' else None
        if adapter:
            _, profile = adapter
            port = profile['cell']['io']['inputs'][profile['input_order'][0]]['positions'][0]
            control = [a+b for a,b in zip(port, profile['hash_origin'])]
            chosen = next((t for t in trials if t['control'] == control), None)
            if chosen is None:
                raise ValueError('Verified adapter control absent from recorded native trials')
        origin = raw['metadata']['origin']
        stem = Path(row['path']).stem
        filename = f'{tier}/{stem}.litematic'
        protocol_file = f'{tier}/{stem}.run.json'
        protocol = {'schema_version': 1, 'source_sha256': key, 'evidence_tier': tier, 'qualification': TIERS[tier],
                    'coordinate_system': 'Original schematic coordinates retained; live controls use local coordinates.',
                    'hash_origin': origin, 'source_data_version': row.get('data_version'),
                    'settle': config['settle'], 'rng_seed': config['rng_seed'], 'setup_ticks': config['setup_ticks'],
                    'trial_ticks': chosen['first']['requested_ticks'], 'control_source': chosen['control'],
                    'control_local': [a-b for a,b in zip(chosen['control'], origin)],
                    'actions': chosen['first']['actions'], 'trace_fingerprint': chosen['first']['trace_fingerprint'],
                    'native_result': str(result_path.relative_to(archive)), 'native_result_sha256': digest(raw_bytes),
                    'review_tags': row.get('review_tags', []), 'selection_edge_contacts': row.get('cutoff_face_contacts', {}),
                    'inventory': row.get('inventory', {}), 'functional_status': status.get('functional_status'),
                    'vanilla_status': status.get('vanilla_status'),
                    'note': 'The recorded native protocol checks motion/replay. Only the attached verified adapter/oracle proves aperture operation.'}
        if adapter:
            directory, profile = adapter
            adapter_folder = f'{stem}-adapter'
            protocol['verified_adapter'] = {'id': profile['id'], 'profile': f'{adapter_folder}/adapter.json', 'suite': f'{adapter_folder}/suite.json',
                                            'setup_ticks': profile['setup_ticks'], 'response_ticks': profile['response_ticks']}
            for name in ['adapter.json', 'suite.json', 'result.json', 'oracle.py', 'mapping.md']:
                p = directory / name
                if p.is_file(): payloads[f'{tier}/{adapter_folder}/{name}'] = p.read_bytes()
        entry = {'source_sha256': key, 'source': row['source'], 'source_path': row['path'],
                 'title': row.get('title') or row.get('display_name') or stem, 'tier': tier,
                 'schematic': filename, 'protocol': protocol_file, 'control_local': protocol['control_local'],
                 'control_source': chosen['control'], 'setup_ticks': config['setup_ticks'],
                 'qualification': TIERS[tier], 'selection_edge_contacts': protocol['selection_edge_contacts']}
        selected.append(entry); payloads[filename] = data; payloads[protocol_file] = encoded(protocol)
    counts = dict(Counter(e['tier'] for e in selected))
    manifest = {'schema_version': 1, 'scope': 'Selection from retained evidence; no fresh simulation or new vanilla certification.',
                'candidate_count': len(rows), 'counts': counts, 'entries': selected, 'excluded': excluded,
                'excluded_counts': dict(Counter(e['reason'] for e in excluded)), 'provenance': provenance}
    readme = ('PLOT DOOR COLLECTION\n\n' + '\n'.join(f'{name}: {counts.get(name, 0)} — {description}' for name,description in TIERS.items())
              + '\n\nThe verified folder is the strict runnable-door set. Candidate folders contain observed motion, not proof of complete doors.\n'
              'Files are byte-identical to the isolated originals, with saved origins, source versions, entities and inventories retained.\n'
              'No missing inventory, missing control or cut-off mechanism was repaired. Edge contacts remain in each run file.\n'
              'Each .run.json names the actual tested control in saved and local coordinates, setup ticks and recorded inputs.\n'
              'Load InWorld, seed 0, run setup ticks, then use the listed control at the recorded times. For the verified door use its adapter and suite for the independent aperture assertions.\n'
              'Raw results stay under validation/plot-native-v3; the hashes identify the retained run, not a new run today.\n'
              'Keep this directory outside Git and back it up separately.\n')
    readme += '\nInstall nucleation==0.12.3, then run: python run_door.py TIER/PART.run.json\nThe script checks source bytes, coordinates, stepping and recorded control transitions. It does not promote candidates to verified doors.\n'
    payloads['run_door.py'] = (REPO/'tools/run_tick_plot_door.py').read_bytes()
    payloads['README.txt'] = readme.encode(); payloads['manifest.json'] = encoded(manifest)
    revision = digest(encoded({**provenance, 'files': {k:digest(v) for k,v in payloads.items()}}))[:16]
    output = archive / 'collections/plot-doors' / revision
    for name,data in payloads.items(): write_identical(output/name, data)
    for bundle, tiers in [('verified-doors', {'verified'}), ('interactive-door-candidates', {'repeatable', 'responsive'})]:
        path = output / f'{bundle}.zip'
        # Deterministic ZIP headers, so reruns validate the exact same bytes.
        import io
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as z:
            for name,data in sorted(payloads.items()):
                if name in {'README.txt', 'manifest.json', 'run_door.py'} or name.split('/')[0] in tiers:
                    info = zipfile.ZipInfo(name, (2026, 11, 15, 0, 0, 0)); info.compress_type = zipfile.ZIP_DEFLATED
                    z.writestr(info, data)
        write_identical(path, buffer.getvalue())
    current = {'schema_version': 1, 'directory': str(output.relative_to(archive)), 'counts': counts,
               'manifest_sha256': digest(payloads['manifest.json']), 'entries': selected,
               'bundles': {name: {'path': str((output/f'{name}.zip').relative_to(archive)), 'sha256': digest((output/f'{name}.zip').read_bytes())}
                           for name in ['verified-doors', 'interactive-door-candidates']}}
    pointer = archive/'collections/plot-doors/current.json'
    temp = pointer.with_suffix('.tmp'); temp.write_bytes(encoded(current)); temp.replace(pointer)
    return {'directory': str(output), 'counts': counts, 'excluded_counts': manifest['excluded_counts'],
            'candidate_count': len(rows), 'schematic_files': len(selected)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', required=True, type=Path)
    parser.add_argument('--native', default='validation/plot-native-v3')
    args = parser.parse_args()
    print(json.dumps(export(args.archive, inside(args.archive, args.native)), indent=2))

if __name__ == '__main__': main()
