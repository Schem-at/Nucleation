#!/usr/bin/env python3
"""Bounded, read-only Circuit Atlas API/data checks; no browser or engine execution.

Run against an existing loopback server. Downloads are compared with the retained
archive, never executed. Static cells/thumbnail checks are not vanilla proofs.
"""
import argparse
import collections
import datetime
import hashlib
import importlib.util
import json
import pathlib
import struct
import sys
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

MAX_RESPONSE = 64 * 1024 * 1024
PINNED = {
    'comparator': '92d44580dc1539903b56ad08a8b1be89d70de602328d5fe36ef589743c3a3db3',
    'latch': '0663302dfc40ac8c33055d5a43aefe834be4290bdda9baaec6f2ccab238cad54',
    'flyer': 'be17be1f79c2bee74f909b4205a3b1ad7c5e9e70c69454abc51b51927fd585dd',
    'plot': '5476ac349cd4529d7a237828633b9e7a995d70b8937f70e4836d2f9f8db66c1e',
    'door': 'e7b964e92109edebd0860f1ec95c158eebd8f66436e4c6c5b7c36862b7675586',
    'legacy': 'a86c62923573e8b65e7e7fde22ebfc80082520496589b49615e1b988923cf1b6',
}
PASS_STATUSES = {'functional_pass', 'vanilla_tick_end_parity_pass'}


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def signed_words(values, bits):
    # Independent whole-stream packing: exercises signed longs and crossings.
    stream = sum(value << (i * bits) for i, value in enumerate(values))
    words = [(stream >> (64 * i)) & ((1 << 64) - 1)
             for i in range((len(values) * bits + 63) // 64)]
    return [v if v < 1 << 63 else v - (1 << 64) for v in words]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-url', default='http://127.0.0.1:8767')
    parser.add_argument('--archive', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    url = urllib.parse.urlparse(args.base_url)
    if url.scheme != 'http' or url.hostname not in {'127.0.0.1', 'localhost', '::1'} or url.path not in {'', '/'}:
        parser.error('Use an HTTP loopback server without a path prefix')
    base = args.base_url.rstrip('/')
    root = args.archive.resolve()
    server_path = pathlib.Path(__file__).with_name('serve_tick_archive.py')
    spec = importlib.util.spec_from_file_location('atlas_under_test', server_path)
    server = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(server)
    requests, checks = [], []

    def request(path, status=200, kind=None):
        try:
            response = urllib.request.urlopen(base + path, timeout=30)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            data = response.read(MAX_RESPONSE + 1)
            actual = response.status
            content_type = response.headers.get('Content-Type', '').split(';')[0]
            require(len(data) <= MAX_RESPONSE, 'Response exceeds64MiB cap')
            requests.append({'path':path, 'status':actual, 'bytes':len(data), 'sha256':sha(data), 'content_type':content_type})
            require(actual in status if isinstance(status, tuple) else actual == status,
                    f'{path}: expected HTTP{status}, got{actual}')
            if kind:require(content_type == kind, f'{path}: unexpected Content-Type {content_type}')
            require(response.headers.get('X-Content-Type-Options') == 'nosniff', f'{path}: missing nosniff')
            return data

    def get_json(path):
        return json.loads(request(path, kind='application/json'))

    def check(name, action):
        try:
            details = action()
            checks.append({'name':name, 'status':'pass', 'details':details})
        except Exception as error:
            checks.append({'name':name, 'status':'fail', 'error':f'{type(error).__name__}: {error}'})

    library = get_json('/api/library')
    rows = library['entries']
    by_hash = collections.defaultdict(list)
    for row in rows:by_hash[row['sha256']].append(row)
    entries = {}

    def entry(digest):
        if digest not in entries:entries[digest] = get_json('/api/entry/'+digest)
        return entries[digest]

    def library_counts():
        original = [r for r in rows if r['source'] not in {'world-record-parts', 'world-record-floor-parts'}]
        expected = {'layout_files':3864, 'validated':4}
        for key, value in expected.items():require(library['summary'][key] == value, f'{key} != {value}')
        schematic = [r for r in rows if r['kind'] == 'schematic']
        original_schematic = [r for r in original if r['kind'] == 'schematic']
        require(len(original_schematic) == 914 and len({r['sha256'] for r in original_schematic}) == 862, 'Original archive changed')
        require(len(schematic) == library['summary']['schematic_files'] and len({r['sha256'] for r in schematic}) == library['summary']['unique_schematics'], 'Summary disagrees with actual schematic rows')
        require(sum(r['kind'] == 'layout' for r in rows) == 3864, 'Layout row count differs')
        validated = {r['sha256'] for r in rows if r.get('validation_status') in PASS_STATUSES}
        require(len(validated) == 4, 'Validated count must be distinct content, not duplicate paths')
        require(len({r['source'] for r in rows}) == library['summary']['sources'], 'Source count differs')
        require(all(r['id'] == r['sha256'] for r in rows), 'Entry ID is not the content hash')
        return {'summary':library['summary'], 'validated_sha256':sorted(validated)}
    check('library counts and distinct validation', library_counts)

    for label in ['comparator', 'flyer', 'legacy', 'plot']:
        def original(label=label):
            digest = PINNED[label]; row = by_hash[digest][0]
            local = (root/row['local_path']).resolve()
            require(local.is_relative_to(root), 'Local manifest path escapes archive')
            data = request('/api/file/'+digest)
            require(sha(data) == digest == sha(local.read_bytes()), 'Original download differs from pinned/local bytes')
            require(len(data) == row['bytes'], 'Original byte count differs')
            return {'source':row['source'], 'path':row['path'], 'sha256':digest, 'bytes':len(data)}
        check('original download '+label, original)

    for label, expected in [('comparator',272), ('latch',25), ('flyer',252)]:
        def preview(label=label, expected=expected):
            result = entry(PINNED[label]); inspection = result['inspection']
            require(inspection['status'] == 'static_preview', 'Expected actual saved-block preview')
            require(inspection['non_air_blocks'] == expected == sum(inspection['materials'].values()), 'Decoded count/material mismatch')
            require('no simulation' in inspection['scope'], 'Preview scope must not imply vanilla proof')
            png = request('/api/thumbnail/'+PINNED[label], kind='image/png')
            require(png[:8] == b'\x89PNG\r\n\x1a\n' and struct.unpack('>II', png[16:24]) == (768,576), 'Expected native 768x576 PNG')
            render = result['entries'][0]['thumbnail']
            require(render['status'] == 'ok' and render['png_sha256'] == sha(png), 'Rendered image hash/provenance mismatch')
            require(render['simulation_ticks'] == 0 and render['non_air_blocks'] == expected, 'Renderer advanced or changed input')
            if label == 'comparator':
                controls = inspection['saved_controls']
                require(len(controls) == 16 and all('powered=false' in x['state'] for x in controls), 'Comparator saved lever properties differ')
                require({tuple(x['pos']) for x in controls} == {(x,1+2*i,10) for x in [0,3] for i in range(8)}, 'Comparator source port coordinates differ')
            return {'non_air_blocks':expected, 'dimensions':inspection['dimensions'], 'png_sha256':sha(png), 'scope':inspection['scope']}
        check('textured saved-state thumbnail '+label, preview)

    def plot_limit():
        result = entry(PINNED['plot']); info = result['inspection']
        require(info['dimensions'] == [284,208,295] and info['bounding_volume'] == 17_426_240, 'Plot dimensions/volume differ')
        require(info['status'] == 'metadata_only' and info['declared_blocks'] == 458409, 'Plot should retain metadata while avoiding full voxel decode')
        require(result['capabilities']['live_sim'] is False and 'limit' in result['capabilities']['reason'], 'Large plot simulation must be gated')
        plot = get_json('/api/plot')
        require(plot['entries'][0]['sha256'] == PINNED['plot'] and len(plot['entries'][0]['candidates']) == 20, 'Plot survey association differs')
        require('projection' not in plot['entries'][0] and 'physical_components_8plus' not in plot['entries'][0], 'Unbounded survey fields should be omitted')
        svg = ET.fromstring(request('/api/plot-map',kind='image/svg+xml'))
        require(svg.tag.endswith('}svg'), 'Plot map is not SVG')
        return {'dimensions':info['dimensions'], 'bounding_volume':info['bounding_volume'], 'live_sim':False, 'candidates':20}
    check('large plot gate and survey association', plot_limit)

    def extracted_parts():
        parts = [r for r in rows if r['source'] == 'world-record-parts']
        require(len(parts) == 893, 'Expected every verified first-pass component')
        require(sum(r['non_air_blocks'] for r in parts) == 458409, 'Source block coverage differs')
        require(sum(r['block_entities'] for r in parts) == 12541, 'Source container/block-entity coverage differs')
        require(sum(r['entities'] for r in parts) == 318, 'Source entity coverage differs')
        doors = [r for r in parts if r['classification'] == 'door_candidate' and 'infrastructure_review' not in r['review_tags']]
        require(len(doors) == 746, 'Shared floors must not count as individual first-pass door candidates')
        require(all(r['data_version'] == 4671 and r.get('validation_status') not in PASS_STATUSES for r in parts), 'Extraction changed version or invented functional evidence')
        require(all(r['thumbnail']['status'] == 'ok' for r in doors), 'An extracted door lacks a verified real render')
        for row in [doors[0], doors[-1]]:
            data = request('/api/file/'+row['sha256'])
            require(sha(data) == row['sha256'], 'Extracted download hash mismatch')
            result = entry(row['sha256'])['entries'][0]
            require(result['verification']['readback'] == 'passed', 'Missing exact source/read-back evidence')
            require(result['derived_from']['sha256'] == PINNED['plot'], 'Extracted source association differs')
        return {'files':len(parts), 'individual_candidates':len(doors), 'source_blocks':458409, 'block_entities':12541, 'entities':318}
    check('extracted plot files, preservation and rendered doors', extracted_parts)

    for label, expected in [('comparator',{'vanilla-comparator':960}), ('door',{'vanilla-cycle':320,'vanilla-idle':320})]:
        def captures(label=label, expected=expected):
            digest = PINNED[label]; result = entry(digest)
            require({c['id']:c['ticks'] for c in result['captures']} == expected, 'Entry capture IDs/tick counts differ')
            observations = []
            for capture in result['captures']:
                require(capture['sha256'] == digest, 'Capture associated with wrong source')
                data = request('/api/capture/'+capture['id'],kind='application/json')
                local = root/'validation/candidate-run'/capture['id']/'capture.json'
                require(sha(data) == sha(local.read_bytes()), 'Capture download differs from retained evidence')
                body = json.loads(data)
                require(body['source']['sha256'] == digest and body['mc_version'] == '1.21.11', 'Capture provenance differs')
                require(body['capture_ticks'] == expected[capture['id']], 'Capture payload tick count differs')
                frames = [body['initial'],*body['snapshots']]
                require([f['tick'] for f in frames] == list(range(body['capture_ticks']+1)), 'Capture frames not complete/ordered')
                require(all(f['players'] == 0 for f in frames), 'Unexpected reference players')
                observations.append({'id':capture['id'], 'frames':len(frames), 'sha256':sha(data)})
            if label == 'door':require(all(r.get('validation_status') not in PASS_STATUSES for r in result['entries']), 'Unsupported door must not count as engine-validated')
            return observations
        check('retained capture association '+label, captures)

    for extension in ['.mcstructure','.txt','.fixture']:
        def unsupported(extension=extension):
            row = next(r for r in rows if pathlib.Path(r['path']).suffix == extension)
            result = entry(row['sha256'])
            require(result['inspection']['status'] == 'metadata_only', 'Unsupported format must remain metadata-only')
            require(result['capabilities']['live_sim'] is False, 'Unsupported format must not advertise live simulation')
            absent = json.loads(request('/api/thumbnail/'+row['sha256'],status=404,kind='application/json'))
            require(absent['render']['status'] != 'ok', 'Unsupported thumbnail must not be replaced by a fake image')
            return {'extension':extension, 'sha256':row['sha256'], 'status':result['inspection']['status'], 'live_sim':False}
        check('metadata-only format '+extension, unsupported)

    def invalid_paths():
        paths = ['/api/file/'+'0'*64, '/api/entry/'+'0'*64, '/api/thumbnail/'+'0'*64,
                 '/api/capture/unknown', '/api/file/%2e%2e%2flibrary.json',
                 '/%2e%2e/tools/serve_tick_archive.py', '/engine/%2e%2e/%2e%2e/Cargo.toml',
                 '/sim-lab/%2e%2e/%2e%2e/Cargo.toml', '/pack/%2e%2e/%2e%2e/Cargo.toml']
        for path in paths:
            body = json.loads(request(path,status=(400,404),kind='application/json'))
            require('error' in body and 'entries' not in body, 'Invalid path returned data instead of an error')
        return {'rejected_paths':len(paths)}
    check('unknown hashes and path traversal rejection', invalid_paths)

    def packed_cells():
        palette = [{'Name':'minecraft:air'}]+[{'Name':f'minecraft:probe_{i}'} for i in range(1,65)]
        palette[1] = {'Name':'minecraft:lever','Properties':{'powered':'false','facing':'west','face':'wall'}}
        values = [(i*17)%65 for i in range(78)]
        words = signed_words(values,7)
        require(any(w<0 for w in words), 'Toy must include signed NBT long words')
        region = {'Position':{'x':-10,'y':5,'z':7}, 'Size':{'x':-13,'y':2,'z':-3},
                  'BlockStatePalette':palette, 'BlockStates':words}
        actual = list(server.cells({'Regions':{'negative':region}}))
        coordinates = [(x,y,z) for y in range(5,7) for z in range(5,8) for x in range(-22,-9)]
        expected = [(coordinates[i], 'minecraft:lever[face=wall,facing=west,powered=false]' if value==1 else f'minecraft:probe_{value}')
                    for i,value in enumerate(values) if value]
        require(actual == expected, 'Signed/negative coordinate or cross-long decode differs')
        require(any((i*7)%64>57 and values[i] for i in range(len(values))), 'Toy does not exercise a cross-long state')
        small = {'Position':{'x':-1,'y':-2,'z':-3},'Size':{'x':2,'y':1,'z':1},
                 'BlockStatePalette':palette[:2], 'BlockStates':signed_words([1,0],2)}
        require(list(server.cells({'Regions':{'tiny':small}})) == [((-1,-2,-3),'minecraft:lever[face=wall,facing=west,powered=false]')], 'Minimum2-bit palette decode differs')
        return {'decoded_cells':len(actual), 'palette_bits':7, 'negative_sizes':[-13,2,-3], 'minimum_palette_bits':2}
    check('toy litematic negative coordinates and packed boundaries', packed_cells)

    def sponge_cells():
        palette = {'minecraft:air':0,'minecraft:stone':1,'minecraft:lever[powered=false]':127,'minecraft:redstone_wire[power=15]':128}
        raw = bytes([0,128|0x80,1,1,127])
        common = {'Width':2,'Height':1,'Length':2}
        v2 = {**common,'Palette':palette,'BlockData':raw}
        v3 = {'Schematic':{**common,'Blocks':{'Palette':palette,'Data':raw}}}
        expected = [((1,0,0),'minecraft:redstone_wire[power=15]'),((0,0,1),'minecraft:stone'),((1,0,1),'minecraft:lever[powered=false]')]
        require(list(server.cells(v2)) == expected == list(server.cells(v3)), 'Sponge varint/order/property decode differs')
        try:list(server.cells({'Regions':{'large':{'Size':{'x':2_000_001,'y':1,'z':1}}}}))
        except ValueError:pass
        else:raise AssertionError('Large volume must reject before accessing missing palette')
        return {'versions':[2,3], 'multibyte_palette_id':128, 'volume_guard_checked':True}
    check('toy Sponge varints and predecode volume gate', sponge_cells)

    passed = sum(c['status']=='pass' for c in checks)
    report = {'schema_version':1, 'status':'pass' if passed==len(checks) else 'fail',
              'checked_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'base_url':base, 'server_source_sha256':sha(server_path.read_bytes()),
              'test_source_sha256':sha(pathlib.Path(__file__).read_bytes()),
              'library_file_sha256':sha((root/'library.json').read_bytes()),
              'checks_passed':passed, 'checks_total':len(checks), 'http_requests':len(requests),
              'scope':'Read-only loopback API, original byte hashes, static decoding and retained capture association. No browser/UI/WASM execution, new vanilla run or new circuit parity proof.',
              'checks':checks, 'requests':requests}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({k:report[k] for k in ['status','checks_passed','checks_total','http_requests']}))
    for result in checks:
        if result['status']=='fail':print(json.dumps(result))
    return 0 if report['status']=='pass' else 1


if __name__ == '__main__':sys.exit(main())
