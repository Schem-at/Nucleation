#!/usr/bin/env python3
"""Build a searchable offline index, static NBT inventory and curator join."""
import argparse
import collections
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import struct
from prepare_tick_logic_archive import NBTReader, MAX_NBT


def inspect(path):
    if path.suffix.lower() == '.mcstructure':
        class BedrockReader(NBTReader):
            def number(self, fmt):
                return struct.unpack('<' + fmt, self.take(struct.calcsize('<' + fmt)))[0]
        _, root = BedrockReader(path.read_bytes()).root()
        palette = root.get('structure', {}).get('palette', {}).get('default', {}).get('block_palette', [])
        return {'inspection': 'Bedrock NBT parsed; Java mechanics not applicable',
                'format': 'mcstructure', 'edition': 'Bedrock', 'data_version': None,
                'dimensions': root.get('size'), 'palette_blocks': sorted({p['name'] for p in palette}),
                'readiness': 'Bedrock; excluded from Java conformance'}
    if path.suffix.lower() == '.snbt':
        match = re.search(r'DataVersion\s*:\s*(\d+)', path.read_text())
        return {'inspection': 'text version only', 'data_version': int(match[1]) if match else None}
    raw = path.read_bytes()
    if raw[:2] == b'\x1f\x8b':
        with gzip.GzipFile(fileobj=io.BytesIO(raw)) as file:
            raw = file.read(MAX_NBT + 1)
    if len(raw) > MAX_NBT:
        return {'inspection': 'decompressed size limit; not parsed'}
    _, root = NBTReader(raw).root()
    result = {'inspection': 'NBT parsed; not simulated'}
    if 'Regions' in root:
        result.update(format='litematic', data_version=root.get('MinecraftDataVersion'),
                      regions=len(root['Regions']), declared_blocks=root.get('Metadata', {}).get('TotalBlocks'))
        materials = set()
        entities = block_entities = 0
        for region in root['Regions'].values():
            materials.update(s['Name'] for s in region['BlockStatePalette'])
            entities += len(region.get('Entities', []))
            block_entities += len(region.get('TileEntities', []))
        result.update(palette_blocks=sorted(materials), entities=entities, block_entities=block_entities)
    else:
        s = root.get('Schematic', root)
        result.update(format='sponge' if 'Version' in s else 'legacy/structure', data_version=s.get('DataVersion'))
        if all(k in s for k in ('Width', 'Height', 'Length')):
            result['dimensions'] = [s[k] for k in ('Width', 'Height', 'Length')]
        palette = s.get('Blocks', s)
        if isinstance(palette, dict) and 'Palette' in palette:
            result['palette_blocks'] = sorted({k.split('[')[0] for k in palette['Palette']})
        result['entities'] = len(s.get('Entities', s.get('entities', [])))
    return result


HTML = '''<!doctype html><html lang="en"><meta charset="utf-8"><title>Nucleation schematic archive</title>
<meta name="viewport" content="width=device-width,initial-scale=1"><style>
body{font:16px system-ui;background:#f4f5f7;color:#172234;margin:0}main{max-width:1450px;margin:auto;padding:32px}
h1{font-size:30px;margin:0 0 8px}p{line-height:1.5;color:#4b5664}input,select,button{font:inherit;padding:10px;border:1px solid #abb5c2;border-radius:6px;background:white}input{min-width:40%}
.filters{display:flex;gap:10px;flex-wrap:wrap;margin:20px 0}.stats{font-weight:600}table{border-collapse:collapse;width:100%;background:white}td,th{text-align:left;padding:12px;border-bottom:1px solid #e0e5eb;vertical-align:top}th{background:#e7edf3}a{color:#164d92}small{display:block;color:#536274;margin-top:6px;overflow-wrap:anywhere}.tag{font-size:12px;background:#e9eff4;padding:3px 6px;margin:2px;display:inline-block;border-radius:4px}.paging{margin:18px 0;display:flex;gap:12px;align-items:center}#empty{padding:20px}
</style><main><h1>Nucleation schematic archive</h1><p>Original files, pinned sources and hashes. Downloading or parsing a file does not establish that its circuit works. Native save versions are preserved; declared versions and curator tags remain separate.</p><div class="stats" id="stats"></div>
<div class="filters"><input id="query" placeholder="Search names, mechanics, blocks or tags" aria-label="Search archive"><select id="source" aria-label="Source"><option value="">All sources</option></select><select id="kind" aria-label="Format"><option value="">Schematics + text layouts</option><option>schematic</option><option>layout</option></select><select id="priority" aria-label="Priority"><option value="">All priorities</option><option>high</option><option>medium</option><option>low</option></select></div>
<p id="count"></p><table><thead><tr><th>Design / original file</th><th>Source / format</th><th>Save version</th><th>Curation / validation</th></tr></thead><tbody id="rows"></tbody></table><div id="empty" hidden>No matching designs.</div><div class="paging"><button id="prev">Previous</button><span id="page"></span><button id="next">Next</button></div><p><a href="library.json">Full machine-readable library</a> · <a href="index.json">Download manifests</a> · <a href="README.md">Archive notes</a></p></main>
<script id="data" type="application/json">DATA</script><script>
const data=JSON.parse(document.getElementById('data').textContent), entries=data.entries; let page=0;
const $=id=>document.getElementById(id), sources=[...new Set(entries.map(e=>e.source))].sort();
for(const s of sources){let o=document.createElement('option');o.value=s;o.textContent=s;$('source').append(o)}
$('stats').textContent=`${data.schematic_files.toLocaleString()} schematic files · ${data.unique_schematics.toLocaleString()} distinct schematic contents · ${data.layout_files.toLocaleString()} text layouts · ${sources.length} sources`;
function link(label,url){let a=document.createElement('a');a.textContent=label;a.href=url;return a}
function render(){let q=$('query').value.toLowerCase();let rows=entries.filter(e=>(!$('source').value||e.source===$('source').value)&&(!$('kind').value||e.kind===$('kind').value)&&(!$('priority').value||e.priority===$('priority').value)&&(!q||e.search.includes(q)));let pages=Math.max(1,Math.ceil(rows.length/100));page=Math.min(page,pages-1);$('rows').replaceChildren();$('count').textContent=`${rows.length.toLocaleString()} matching entries`;$('empty').hidden=!!rows.length;
for(const e of rows.slice(page*100,(page+1)*100)){let tr=document.createElement('tr');let cells=Array.from({length:4},()=>document.createElement('td'));cells[0].append(link(e.title||e.path.split('/').pop(),e.local_path.split('/').map(encodeURIComponent).join('/')));let p=document.createElement('small');p.textContent=e.path;cells[0].append(p);if(e.documentation){cells[0].append(link('Design notes',e.documentation.split('/').map(encodeURIComponent).join('/')))}cells[1].append(link(e.source,e.source_url));let f=document.createElement('small');f.textContent=`${e.kind} · ${(e.bytes/1024).toFixed(1)} KiB · ${e.copies} provenance entries with these bytes`;cells[1].append(f);cells[2].textContent=e.data_version??'Not stamped / not parsed';cells[3].textContent=e.validation_status||e.readiness||'Unvalidated';for(const tag of (e.tags||[])){let s=document.createElement('span');s.className='tag';s.textContent=tag;cells[3].append(s)}let why=document.createElement('small');why.textContent=e.test_plan||e.evidence_basis||e.inspection||'Hash-verified download';cells[3].append(why);tr.append(...cells);$('rows').append(tr)}$('page').textContent=`Page ${page+1} / ${pages}`;$('prev').disabled=page===0;$('next').disabled=page+1===pages;}
for(const id of ['query','source','kind','priority'])$(id).addEventListener('input',()=>{page=0;render()});$('prev').onclick=()=>{page--;render()};$('next').onclick=()=>{page++;render()};render();
</script></html>'''


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--archive', type=Path, required=True)
    a = p.parse_args()
    root = a.archive.resolve()
    cached_path = root / 'inspection-cache.json'
    cached = json.loads(cached_path.read_text()) if cached_path.exists() else {}
    curated = {}
    for path in sorted((root / 'curation').glob('*.json')):
        for row in json.loads(path.read_text()).get('entries', []):
            curated[row['source'], row['path']] = row
    validation_path = root / 'validation/status.json'
    validation = json.loads(validation_path.read_text()) if validation_path.exists() else {}
    entries = []
    for manifest_path in sorted((root / 'sources').glob('*/manifest.json')):
        manifest = json.loads(manifest_path.read_text())
        source = manifest['source']
        for r in manifest['files']:
            if r['status'] != 'downloaded' or r['kind'] not in ('schematic', 'layout'):
                continue
            row = dict(r, source=source['id'], source_url=r.get('source_url', source['url']))
            if row['kind'] == 'schematic':
                if row['sha256'] not in cached or cached[row['sha256']].get('inspection_error'):
                    try:
                        cached[row['sha256']] = inspect(root / row['local_path'])
                    except Exception as error:
                        cached[row['sha256']] = {'inspection': 'not parsed', 'inspection_error': str(error)}
                row.update(cached[row['sha256']])
            key = row['source'], row['path']
            row.update(curated.get(key, {}))
            row.update(validation.get(row['sha256'], {}))
            local = root / row['local_path']
            for folder in (local.parent, local.parent.parent):
                if (folder / 'README.md').exists():
                    row['documentation'] = str((folder / 'README.md').relative_to(root)); break
            row['search'] = json.dumps(row, ensure_ascii=False).lower()
            entries.append(row)
    counts = collections.Counter(row['sha256'] for row in entries)
    for row in entries:
        row['copies'] = counts[row['sha256']]
    entries.sort(key=lambda r: ({'high':0,'medium':1,'low':3}.get(r.get('priority'),2),r['source'],r['path']))
    result = {'schematic_files': sum(r['kind'] == 'schematic' for r in entries),
              'unique_schematics': len({r['sha256'] for r in entries if r['kind']=='schematic'}),
              'layout_files': sum(r['kind']=='layout' for r in entries), 'entries': entries,
              'unmatched_curation': [list(key) for key in curated if key not in {(r['source'],r['path']) for r in entries}]}
    cached_path.write_text(json.dumps(cached, indent=2) + '\n')
    (root / 'library.json').write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    (root / 'index.html').write_text(HTML.replace('DATA', json.dumps(result,ensure_ascii=False).replace('<','\\u003c')))
    print(json.dumps({k:v for k,v in result.items() if k!='entries'}))


if __name__ == '__main__':
    main()
