#!/usr/bin/env python3
"""Cache saved Mojang head skins outside Git and optionally make a render pack.

No player/account lookup: only texture URLs actually embedded in the source NBT.
The renderer remains offline; this tool supplies ordinary resource-pack textures.
"""
import argparse
import base64
import concurrent.futures
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import subprocess
import tempfile
import urllib.parse
import urllib.request
import zipfile

from prepare_tick_logic_archive import NBTReader, MAX_NBT

HASH = re.compile(r'[a-f0-9]{1,64}')
MAX_SKIN = 256 * 1024


def skin_hash(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme not in {'http', 'https'} or parsed.hostname != 'textures.minecraft.net'
            or parsed.username or parsed.password or parsed.query or parsed.fragment
            or parsed.port not in {None, 80 if parsed.scheme == 'http' else 443}):
        raise ValueError('Expected a saved textures.minecraft.net skin URL')
    match = re.fullmatch(r'/texture/([a-fA-F0-9]{1,64})', parsed.path)
    if not match: raise ValueError('Invalid skin texture hash')
    return match[1].lower()


def validate_png(data):
    if (len(data) < 33 or len(data) > MAX_SKIN or data[:8] != b'\x89PNG\r\n\x1a\n'
            or data[12:16] != b'IHDR' or struct.unpack('>II', data[16:24]) not in {(64, 64), (64, 32)}):
        raise ValueError('Expected a bounded 64x64 or legacy 64x32 skin PNG')
    return data


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        raise ValueError('Skin CDN redirects are not accepted')


def cached_skin(directory, digest):
    if not HASH.fullmatch(digest): raise ValueError('Invalid skin texture hash')
    directory = Path(directory)
    path = directory / f'{digest}.png'
    if path.is_file():
        validate_png(path.read_bytes())
        return path
    request = urllib.request.Request(f'https://textures.minecraft.net/texture/{digest}',
                                     headers={'User-Agent': 'Nucleation-Circuit-Atlas'})
    with urllib.request.build_opener(NoRedirect).open(request, timeout=6) as response:
        data = validate_png(response.read(MAX_SKIN + 1))
    directory.mkdir(parents=True, exist_ok=True)
    # Independent temporary files also make concurrent requests for one skin safe.
    with tempfile.NamedTemporaryFile(dir=directory, delete=False) as tmp:
        tmp.write(data); temporary = Path(tmp.name)
    temporary.replace(path)
    return path


def profile_urls(value):
    """Read the saved texture property from old and current profile encodings."""
    if not isinstance(value, dict): return set()
    urls = set()
    for key in ('SkullOwner', 'profile', 'Profile'):
        profile = value.get(key)
        if not isinstance(profile, dict) or profile.get('texture'): continue
        properties = profile.get('properties', profile.get('Properties', {}))
        if isinstance(properties, dict): entries = properties.get('textures', [])
        elif isinstance(properties, list): entries = [p for p in properties if isinstance(p, dict) and p.get('name') == 'textures']
        else: entries = []
        for prop in entries:
            if not isinstance(prop, dict): continue
            try:
                encoded = prop.get('value', prop.get('Value', ''))
                decoded = json.loads(base64.urlsafe_b64decode(encoded + '=' * (-len(encoded) % 4)))
                url = decoded['textures']['SKIN']['url']
                digest = skin_hash(url)
                urls.add(f'https://textures.minecraft.net/texture/{digest}')
            except (ValueError, KeyError, TypeError): pass
    return urls


def source_urls(path):
    raw = path.read_bytes()
    if raw[:2] == b'\x1f\x8b':
        with gzip.GzipFile(fileobj=io.BytesIO(raw)) as gz: raw = gz.read(MAX_NBT + 1)
    if len(raw) > MAX_NBT: raise ValueError('NBT exceeds archive decode limit')
    _, root = NBTReader(raw).root()
    if 'Regions' in root:
        entities = [be for region in root['Regions'].values() for be in region.get('TileEntities', [])]
    else:
        root = root.get('Schematic', root)
        entities = root.get('Blocks', root).get('BlockEntities', root.get('TileEntities', []))
        if not entities: entities = [b.get('nbt', {}) for b in root.get('blocks', [])]
    return sorted({url for be in entities for url in profile_urls(be.get('Data', be))})


def make_pack(base, output, skins):
    if base.resolve() == output.resolve(): raise ValueError('Keep the original pack intact')
    if output.exists(): raise ValueError('Choose a new output pack; existing packs retain their provenance')
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(base) as source, zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as target:
        for name in source.namelist():
            if name.startswith('assets/nucleation/textures/skins/'): continue
            target.writestr(source.getinfo(name), source.read(name))
        for path in sorted(skins):
            data = path.read_bytes()
            if struct.unpack('>II', data[16:24]) == (64, 32):
                from PIL import Image
                # Same legacy padding as ResourcePackSource::add_player_skin.
                padded = Image.new('RGBA', (64, 64))
                padded.paste(Image.open(io.BytesIO(data)).convert('RGBA'), (0, 0))
                buffer = io.BytesIO(); padded.save(buffer, format='PNG'); data = buffer.getvalue()
            info = zipfile.ZipInfo(f'assets/nucleation/textures/skins/{path.name}', (2020, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            target.writestr(info, data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--pack', type=Path)
    parser.add_argument('--output-pack', type=Path)
    args = parser.parse_args()
    if bool(args.pack) != bool(args.output_pack): parser.error('--pack and --output-pack belong together')
    root = args.archive.resolve()
    if subprocess.run(['git', '-C', str(root), 'rev-parse', '--show-toplevel'], capture_output=True).returncode == 0:
        raise ValueError('Keep cached skins and schematic archives outside Git')
    rows = {r['sha256']: r for r in json.loads((root/'library.json').read_bytes())['entries']
            if Path(r['path']).suffix != '.mcstructure'
            and {'minecraft:player_head', 'minecraft:player_wall_head'} & set(r.get('palette_blocks', []))}
    sources, errors = {}, {}
    for digest, row in sorted(rows.items()):
        try:
            path = (root/row['local_path']).resolve()
            if not path.is_relative_to(root): raise ValueError('Source path escapes archive')
            if hashlib.sha256(path.read_bytes()).hexdigest() != digest: raise ValueError('Source hash changed')
            sources[digest] = {'local_path': row['local_path'], 'urls': source_urls(path)}
        except Exception as error: errors[digest] = str(error)
    hashes = sorted({skin_hash(url) for s in sources.values() for url in s['urls']})
    cache = root/'player-skins'; results = {}
    def download(digest):
        try:
            path = cached_skin(cache, digest)
            return digest, {'status':'ok', 'png_sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
        except Exception as error: return digest, {'status':'unavailable', 'error':str(error)}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        results.update(pool.map(download, hashes))
    report = {'sources':sources, 'source_errors':errors, 'skins':results,
              'scope':'Saved texture URLs only; original schematics are unchanged.'}
    if args.pack:
        make_pack(args.pack, args.output_pack, [cache/f'{h}.png' for h,r in results.items() if r['status']=='ok'])
        report['pack'] = {'base_sha256':hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                          'output_sha256':hashlib.sha256(args.output_pack.read_bytes()).hexdigest(),
                          'output':str(args.output_pack)}
    cache.mkdir(parents=True, exist_ok=True)
    (cache/'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({'head_sources':len(sources), 'source_errors':errors, 'skins':len(results),
                      'loaded':sum(r['status']=='ok' for r in results.values()),
                      'unavailable':{h:r for h,r in results.items() if r['status']!='ok'}, 'pack':report.get('pack')}))

if __name__ == '__main__': main()
