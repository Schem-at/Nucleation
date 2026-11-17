#!/usr/bin/env python3
"""Loopback Circuit Atlas server over retained originals and evidence.

python3 tools/serve_tick_archive.py --archive /path/to/archive --port 8767
Rust/WASM builds are separate; this server never compiles or executes downloads.
Saved Mojang skin PNGs are cached separately without changing any original.
"""
import argparse
import collections
import gzip
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import io
import json
import mimetypes
from pathlib import Path
import re
import threading
import urllib.parse
from xml.sax.saxutils import escape

from prepare_tick_logic_archive import NBTReader, MAX_NBT, varints
from player_head_skins import cached_skin

REPO = Path(__file__).resolve().parents[1]
APP = REPO / 'apps/schematic-atlas'
LAB = REPO / 'apps/sim-lab-wasm'
AIR = {'minecraft:air', 'minecraft:cave_air', 'minecraft:void_air'}
MAX_CELLS = 2_000_000


def descriptor(state):
    props = state.get('Properties', {})
    return state['Name'] + ('[' + ','.join(f'{k}={v}' for k, v in sorted(props.items())) + ']' if props else '')


def cells(root):
    """Yield saved coordinates/descriptors; no placement, datafixing or simulation."""
    if 'Regions' in root:
        total = sum(abs(r['Size']['x'] * r['Size']['y'] * r['Size']['z']) for r in root['Regions'].values())
        if total > MAX_CELLS:
            raise ValueError('Large selection: use its survey map or explicit live load; automatic voxel preview is bounded.')
        for region in root['Regions'].values():
            size = [abs(region['Size'][a]) for a in 'xyz']
            base = [region['Position'][a] + min(0, region['Size'][a] + 1) for a in 'xyz']
            palette = [descriptor(s) for s in region['BlockStatePalette']]
            bits = max(2, (len(palette) - 1).bit_length())
            words = [w & ((1 << 64) - 1) for w in region['BlockStates']]
            sx, sy, sz = size
            for index in range(sx * sy * sz):
                word, offset = divmod(index * bits, 64)
                value = words[word] >> offset
                if offset + bits > 64:
                    value |= words[word + 1] << (64 - offset)
                state = palette[value & ((1 << bits) - 1)]
                if state.split('[', 1)[0] not in AIR:
                    yield (base[0] + index % sx, base[1] + index // (sx * sz), base[2] + index // sx % sz), state
    else:
        root = root.get('Schematic', root)
        if not all(k in root for k in ['Width', 'Height', 'Length']):
            raise ValueError('This format has no automatic voxel preview; original and metadata are available.')
        sx, sy, sz = [root[k] for k in ['Width', 'Height', 'Length']]
        if sx * sy * sz > MAX_CELLS:
            raise ValueError('Large selection: automatic voxel preview is bounded.')
        blocks = root.get('Blocks', root)
        if 'Palette' not in blocks:
            raise ValueError('Legacy numeric IDs require production conversion before a named-block preview.')
        palette = {v: k for k, v in blocks['Palette'].items()}
        for i, value in enumerate(varints(blocks.get('Data', blocks.get('BlockData')), sx * sy * sz)):
            state = palette[value]
            if state.split('[', 1)[0] not in AIR:
                yield (i % sx, i // (sx * sz), i // sx % sz), state


def color(name):
    if 'piston' in name or 'slime' in name: return '#b9d86d'
    if any(k in name for k in ['redstone', 'repeater', 'comparator', 'observer']): return '#ec725f'
    if any(k in name for k in ['hopper', 'chest', 'dropper', 'dispenser', 'barrel', 'crafter']): return '#76a9be'
    if any(k in name for k in ['lever', 'button', 'torch', 'lamp']): return '#e8c56f'
    if 'glass' in name: return '#b3c9c5'
    return '#606b67'


class Archive:
    def __init__(self, root):
        self.root = root.resolve()
        self.library = json.loads((root / 'library.json').read_text())
        self.by_hash = collections.defaultdict(list)
        for row in self.library['entries']:
            self.by_hash[row['sha256']].append(row)
        self.door_exports = {}
        pointer = root / 'collections/plot-doors/current.json'
        if pointer.is_file():
            current = json.loads(pointer.read_bytes())
            directory = (root / current['directory']).resolve()
            if not directory.is_relative_to(root.resolve()):
                raise ValueError('Door export path escapes archive')
            data = (directory / 'manifest.json').read_bytes()
            manifest = json.loads(data)
            if (hashlib.sha256(data).hexdigest() != current['manifest_sha256']
                    or current['entries'] != manifest['entries'] or current['counts'] != manifest['counts']):
                raise ValueError('Door collection manifest changed; regenerate its export')
            self.door_exports = current
            for entry in current['entries']:
                for row in self.by_hash.get(entry['source_sha256'], []):
                    row['door_export'] = entry
        self.inspections = {}
        render_manifest = root / 'rendered-thumbnails/manifest.json'
        self.thumbnails = json.loads(render_manifest.read_text()).get('entries', {}) if render_manifest.exists() else {}
        self.lock = threading.Lock()
        self.work = threading.BoundedSemaphore(3)
        self.adapters = {}
        for path in sorted((REPO / 'circuit-adapters').glob('*/adapter.json')):
            profile = json.loads(path.read_text())
            result_path = path.with_name('result.json')
            result = json.loads(result_path.read_text()) if result_path.exists() else None
            suite = json.loads(path.with_name('suite.json').read_text())
            if result and any(result.get(key) != value for key, value in {
                'source_sha256': profile['source_sha256'],
                'profile_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                'suite_sha256': hashlib.sha256(path.with_name('suite.json').read_bytes()).hexdigest(),
            }.items()):
                result = {'status': 'stale_result', 'note': 'Adapter or suite changed since this run.'}
            self.adapters[profile['source_sha256']] = {
                'id': path.parent.name, 'profile': profile, 'vectors': len(suite['vectors']),
                'oracle': suite['oracle'], 'result': result,
                'oracle_code': path.with_name('oracle.py').is_file(),
                'mapping_notes': path.with_name('mapping.md').is_file(),
            }
        self.captures = {}
        for label, folder in [('Comparator · vanilla', 'vanilla-comparator'), ('Interlocking door · two cycles', 'vanilla-cycle'), ('Interlocking door · idle control', 'vanilla-idle')]:
            path = root / 'validation/candidate-run' / folder / 'capture.json'
            if path.exists():
                data = json.loads(path.read_text())
                self.captures[folder] = {'id': folder, 'label': label, 'ticks': data['capture_ticks'], 'source': 'Minecraft 1.21.11 capture', 'sha256': data['source']['sha256'], 'path': path}
        plot_report = REPO / 'docs/performance/schematic-candidates/plot-validation/oracle-results.json'
        if plot_report.exists():
            for case in json.loads(plot_report.read_text()).get('cases', []):
                folder = (root / 'validation/plot-oracles-v1' / case['artifact_directory']).resolve()
                if not folder.is_relative_to(root.resolve()): continue
                for mode in ['cycle', 'idle']:
                    path = folder / mode / 'capture.json'
                    if not path.exists(): continue
                    capture_id = case['case'] + '-' + mode
                    self.captures[capture_id] = {'id': capture_id, 'label': f"{case['case']} · vanilla {mode}",
                        'ticks': case['captures'][mode]['ticks'], 'source': 'Minecraft 1.21.11 capture',
                        'sha256': case['source_sha256'], 'path': path}

    def path(self, sha):
        if not re.fullmatch(r'[a-f0-9]{64}', sha) or sha not in self.by_hash:
            raise KeyError('Unknown archive content')
        path = (self.root / self.by_hash[sha][0]['local_path']).resolve()
        if not path.is_relative_to(self.root): raise ValueError('Invalid manifest path')
        return path

    def engine(self):
        p = LAB / 'public/engine/build-info.json'
        return json.loads(p.read_text()) if p.exists() else {'status': 'not built'}

    def door_file(self, kind, key):
        if kind == 'bundle':
            entry = self.door_exports.get('bundles', {}).get(key)
            if not entry: raise KeyError('Unknown door bundle')
            path = (self.root / entry['path']).resolve()
            if not path.is_relative_to(self.root.resolve()): raise ValueError('Invalid bundle path')
            if hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
                raise ValueError('Door bundle changed; regenerate its export')
            return path
        if kind == 'protocol':
            entry = next((e for e in self.door_exports.get('entries', []) if e['source_sha256'] == key), None)
            if not entry: raise KeyError('Unknown door protocol')
            path = (self.root / self.door_exports['directory'] / entry['protocol']).resolve()
            if not path.is_relative_to(self.root.resolve()): raise ValueError('Invalid protocol path')
            return path
        raise KeyError('Unknown door file')

    def thumbnail(self, sha):
        result = self.thumbnails.get(sha, {'status': 'missing'})
        if result.get('status') == 'ok' and not (self.root / 'rendered-thumbnails/pngs' / f'{sha}.png').is_file():
            return {'status': 'missing'}
        return result

    def inspect(self, sha):
        with self.lock:
            if sha in self.inspections: return self.inspections[sha]
        with self.work:
            result = {}
            try:
                path = self.path(sha)
                if path.suffix.lower() in {'.snbt', '.mcstructure', '.txt', '.fixture'}:
                    raise ValueError('Metadata-only preview for this format; see original file and curator notes.')
                data = path.read_bytes()
                if data[:2] == b'\x1f\x8b':
                    with gzip.GzipFile(fileobj=io.BytesIO(data)) as f: data = f.read(MAX_NBT + 1)
                if len(data) > MAX_NBT: raise ValueError('NBT decompression limit')
                _, root = NBTReader(data).root()
                if 'Regions' in root:
                    regions = list(root['Regions'].values())
                    lo = [min(r['Position'][a] + min(0, r['Size'][a] + 1) for r in regions) for a in 'xyz']
                    hi = [max(r['Position'][a] + max(0, r['Size'][a] - 1) for r in regions) for a in 'xyz']
                    result.update(dimensions=[b-a+1 for a,b in zip(lo,hi)], data_version=root.get('MinecraftDataVersion'),
                                  declared_blocks=root.get('Metadata', {}).get('TotalBlocks'),
                                  entities=sum(len(r.get('Entities', [])) for r in regions),
                                  block_entities=sum(len(r.get('TileEntities', [])) for r in regions))
                else:
                    s = root.get('Schematic', root)
                    section = s.get('Blocks', s)
                    if isinstance(section, dict):
                        result.update(block_entities=len(section.get('BlockEntities', s.get('BlockEntities', s.get('TileEntities', [])))),
                                      entities=len(s.get('Entities', s.get('entities', []))))
                    if all(k in s for k in ['Width','Height','Length']):
                        result.update(dimensions=[s[k] for k in ['Width','Height','Length']], data_version=s.get('DataVersion'))
                if result.get('dimensions'):
                    x,y,z = result['dimensions']; result['bounding_volume'] = x*y*z
                counts, top, controls = collections.Counter(), {}, []
                lower, upper = [2**31] * 3, [-2**31] * 3
                for pos, state in cells(root):
                    name = state.split('[', 1)[0]; counts[name] += 1
                    for a in range(3): lower[a] = min(lower[a], pos[a]); upper[a] = max(upper[a], pos[a])
                    key = pos[0], pos[2]
                    if key not in top or pos[1] >= top[key][0]: top[key] = (pos[1], name)
                    if (name.endswith(':lever') or name.endswith('_button')) and len(controls) < 100:
                        controls.append({'pos': pos, 'state': state})
                result.update({'status': 'static_preview', 'non_air_blocks': sum(counts.values()), 'materials': dict(counts.most_common()),
                          'bounds': {'min': lower, 'max': upper}, 'saved_controls': controls,
                          'scope': 'Saved cells only; no simulation or completeness proof. Multi-region overlaps may be counted more than once.'})
                if top:
                    dx, dz = upper[0] - lower[0] + 1, upper[2] - lower[2] + 1
                    scale = min(280 / max(dx, 1), 190 / max(dz, 1))
                    parts = ['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 240" role="img" aria-label="Saved block projection from above"><rect width="320" height="240" fill="#202925"/>']
                    # Bounded projection; retain the top saved cell per output pixel.
                    pixels = {}
                    for (x, z), (_, name) in top.items(): pixels[(int((x-lower[0])*scale), int((z-lower[2])*scale))] = name
                    ox, oz = (320 - dx * scale) / 2, (214 - dz * scale) / 2
                    for (x, z), name in pixels.items(): parts.append(f'<rect x="{ox+x:.2f}" y="{max(4,oz)+z:.2f}" width="{max(1,scale):.2f}" height="{max(1,scale):.2f}" fill="{color(name)}"/>')
                    parts.append('<text x="12" y="228" font-family="monospace" font-size="10" fill="#a8b6aa">SAVED BLOCKS · TOP VIEW</text></svg>')
                    result['_svg'] = ''.join(parts)
            except Exception as error:
                result.update(status='metadata_only', note=str(error))
            with self.lock: self.inspections[sha] = result
            return result

    def slim(self):
        keys = ['source', 'source_url', 'path', 'title', 'kind', 'bytes', 'sha256', 'data_version', 'dimensions', 'declared_blocks', 'non_air_blocks', 'entities', 'block_entities', 'palette_blocks', 'tags', 'priority', 'readiness', 'validation_status', 'validation_detail', 'validation_evidence', 'simulation_status', 'functional_status', 'vanilla_status', 'benchmark_status', 'local_path', 'evidence_basis', 'inspection', 'category', 'classification', 'bounds', 'cutoff_face_contacts', 'review_tags', 'isolation', 'door_export']
        rows = []
        for row in self.library['entries']:
            record = {k: row[k] for k in keys if k in row}
            record['id'] = row['sha256']
            record['thumbnail'] = self.thumbnail(row['sha256'])
            record['title'] = str(row.get('title') or row.get('name') or Path(row['path']).stem.replace('_', ' '))
            static = row.get('static', {})
            record['non_air_blocks'] = row.get('non_air_blocks', static.get('non_air_blocks', static.get('non_air_region_sum', row.get('declared_blocks'))))
            record['block_entities'] = row.get('block_entities', static.get('block_entity_count', static.get('block_entities')))
            record['entities'] = row.get('entities', static.get('entity_count', static.get('entities')))
            if row['source'] == 'user-world-record-plot':
                record['dimensions'] = [284, 208, 295]
            rows.append(record)
        summary = {k: v for k, v in self.library.items() if k not in ['entries', 'unmatched_curation']}
        summary.update(sources=len({r['source'] for r in rows}), validated=len({r['sha256'] for r in rows if r.get('validation_status') in ['functional_pass', 'vanilla_tick_end_parity_pass', 'vanilla_sampled_block_parity_pass']}))
        return {'summary': summary, 'entries': rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--port', type=int, default=8767)
    args = parser.parse_args()
    archive = Archive(args.archive)

    class Handler(BaseHTTPRequestHandler):
        def send(self, body, kind='application/json; charset=utf-8', status=200, filename=None):
            if not isinstance(body, bytes): body = body.encode()
            self.send_response(status); self.send_header('Content-Type', kind)
            self.send_header('Content-Length', str(len(body))); self.send_header('Cache-Control', 'no-cache')
            self.send_header('X-Content-Type-Options', 'nosniff')
            if filename: self.send_header('Content-Disposition', "inline; filename*=UTF-8''" + urllib.parse.quote(filename, safe=''))
            self.end_headers(); self.wfile.write(body)

        def json(self, obj, status=200): self.send(json.dumps(obj, ensure_ascii=False), status=status)

        def file(self, path, filename=None):
            if not path.is_file(): return self.json({'error': 'File not available'}, 404)
            self.send(path.read_bytes(), mimetypes.guess_type(path)[0] or 'application/octet-stream', filename=filename)

        def do_GET(self):
            path = urllib.parse.unquote(urllib.parse.urlparse(self.path).path)
            try:
                if path == '/api/library': return self.json(archive.slim())
                if path == '/api/engine': return self.json(archive.engine())
                if path.startswith('/api/player-skin/'):
                    digest = path.removeprefix('/api/player-skin/')
                    if not re.fullmatch(r'[a-f0-9]{1,64}', digest): return self.json({'error':'Invalid skin hash'}, 400)
                    try:
                        return self.file(cached_skin(archive.root/'player-skins', digest))
                    except Exception:
                        return self.json({'error':'Saved skin is unavailable; the renderer can use its default skin'}, 502)
                if path.startswith('/api/door-bundle/'):
                    p = archive.door_file('bundle', path.rsplit('/', 1)[1]); return self.file(p, p.name)
                if path.startswith('/api/door-protocol/'):
                    p = archive.door_file('protocol', path.rsplit('/', 1)[1]); return self.file(p, p.name)
                if path.startswith('/api/file/'):
                    p = archive.path(path.rsplit('/', 1)[1]); return self.file(p, p.name)
                if path.startswith('/api/entry/'):
                    sha = path.rsplit('/', 1)[1]; archive.path(sha)
                    rows = [dict({k: v for k,v in r.items() if k not in ['search', 'projection', 'physical_components_8plus']}, thumbnail=archive.thumbnail(sha)) for r in archive.by_hash[sha]]
                    inspection = {k: v for k,v in archive.inspect(sha).items() if k != '_svg'}
                    too_large = inspection.get('bounding_volume', 0) > 8_000_000
                    format_supported = rows[0]['kind'] == 'schematic' and Path(rows[0]['path']).suffix != '.mcstructure'
                    available = (LAB / 'dist/index.html').exists() and (LAB / 'public/engine/nucleation.wasm').exists() and not too_large and format_supported
                    reason = 'This selection exceeds the browser engine limit of 8M cells. Survey individual candidates before extracting complete fixtures.' if too_large else ('This format needs conversion to a supported Java schematic before simulation.' if not format_supported else '')
                    return self.json({'entries': rows, 'inspection': inspection,
                                      'adapter': archive.adapters.get(sha),
                                      'captures': [{k: v for k,v in c.items() if k != 'path'} for c in archive.captures.values() if c['sha256'] == sha],
                                      'capabilities': {'live_sim': available, 'reason': reason}, 'engine': archive.engine()})
                if path.startswith('/api/adapter-file/'):
                    parts = path.split('/')
                    if len(parts) != 5: return self.json({'error':'Unknown adapter file'}, 404)
                    _, _, _, adapter_id, name = parts
                    if adapter_id not in {v['id'] for v in archive.adapters.values()}:
                        return self.json({'error':'Unknown adapter'}, 404)
                    files = {'adapter.json': REPO / 'circuit-adapters' / adapter_id / 'adapter.json',
                             'suite.json': REPO / 'circuit-adapters' / adapter_id / 'suite.json',
                             'result.json': REPO / 'circuit-adapters' / adapter_id / 'result.json',
                             'runner.rs': REPO / 'examples/run_tick_adapter.rs',
                             'implementation.rs': REPO / 'src/tick_adapter.rs',
                             'oracle.py': REPO / 'circuit-adapters' / adapter_id / 'oracle.py',
                             'mapping.md': REPO / 'circuit-adapters' / adapter_id / 'mapping.md',
                             'README.md': REPO / 'circuit-adapters/README.md'}
                    if name not in files: return self.json({'error':'Unknown adapter file'}, 404)
                    return self.file(files[name], name)
                if path.startswith('/api/capture/'):
                    capture = archive.captures.get(path.rsplit('/', 1)[1])
                    if not capture: return self.json({'error': 'No retained capture'}, 404)
                    return self.file(capture['path'])
                if path.startswith('/api/thumbnail/'):
                    sha = path.rsplit('/', 1)[1]; archive.path(sha)
                    if archive.thumbnail(sha).get('status') != 'ok':
                        return self.json({'error': 'No rendered thumbnail', 'render': archive.thumbnail(sha)}, 404)
                    return self.file(archive.root / 'rendered-thumbnails/pngs' / f'{sha}.png')
                if path.startswith('/api/projection/'):
                    sha = path.rsplit('/', 1)[1]; archive.path(sha)
                    svg = archive.inspect(sha).get('_svg')
                    if not svg: return self.json({'error': 'No saved-cell projection'}, 404)
                    return self.send(svg, 'image/svg+xml')
                if path == '/api/plot-map': return self.file(REPO / 'docs/performance/schematic-candidates/world-record-plot-map.svg')
                if path == '/api/plot':
                    p = archive.root / 'curation/world-record-plot.json'
                    d = json.loads(p.read_text())
                    d['entries'] = [{k:v for k,v in r.items() if k not in ['projection','physical_components_8plus']} for r in d.get('entries', [])]
                    return self.json(d)
                for prefix, folder in [('/engine/', LAB/'public/engine'), ('/pack/', LAB/'public/pack'), ('/sim-lab/', LAB/'dist')]:
                    if path.startswith(prefix):
                        relative = path[len(prefix):] or 'index.html'
                        resolved = (folder / relative).resolve()
                        if not resolved.is_relative_to(folder.resolve()): return self.json({'error': 'Invalid path'}, 400)
                        return self.file(resolved)
                relative = path.lstrip('/') or 'index.html'; target = (APP / relative).resolve()
                if not target.is_relative_to(APP.resolve()): return self.json({'error': 'Invalid path'}, 400)
                return self.file(target)
            except (KeyError, ValueError) as error: self.json({'error': str(error)}, 404)
            except (OSError, json.JSONDecodeError) as error: self.json({'error': str(error)}, 500)

        def log_message(self, fmt, *args):
            if args and str(args[1] if len(args)>1 else '') not in ['200', '304']: super().log_message(fmt, *args)

    server = ThreadingHTTPServer(('127.0.0.1', args.port), Handler)
    print(f'Circuit Atlas: http://127.0.0.1:{args.port} — {len(archive.by_hash)} distinct archived contents', flush=True)
    try: server.serve_forever()
    except KeyboardInterrupt: pass
    finally: server.server_close()


if __name__ == '__main__': main()
