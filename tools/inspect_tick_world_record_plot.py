#!/usr/bin/env python3
"""Read-only bounded NBT survey of a user-supplied WorldRecordPlot.litematic.

No simulation, exports, world edits or downloads. Output is metadata: exact
saved blocks, container-field observations, boundary contacts and provisional
six-neighbour component candidates. Component connectivity does not prove a
complete circuit; moving geometry, diagonal power/QC and shared infrastructure
can cross a component boundary. Source inventory loss is never inferred merely
from absent/empty saved Items tags.
"""
import argparse
import collections
import gzip
import hashlib
import importlib.util
import io
import json
import math
import pathlib

MAX_NBT = 128 * 1024 * 1024
MAX_CELLS = 25_000_000
MAX_NONAIR = 1_000_000
CONTAINERS = {'hopper', 'dropper', 'dispenser', 'chest', 'trapped_chest', 'barrel',
              'furnace', 'blast_furnace', 'smoker', 'brewing_stand', 'crafter',
              'campfire', 'soul_campfire', 'decorated_pot', 'chiseled_bookshelf', 'shelf'}


def short(name):
    return name.removeprefix('minecraft:')


def container(name):
    return short(name) in CONTAINERS or short(name).endswith(('shulker_box', '_shelf'))


def control(name):
    return short(name) == 'lever' or short(name).endswith('_button')


def mechanism(name):
    n = short(name)
    return (container(name) or control(name) or any(s in n for s in
            ['piston', 'observer', 'redstone', 'repeater', 'comparator', 'rail',
             'slime_block', 'honey_block', 'target', 'sculk', 'copper_bulb']))


def nbt_helpers():
    source = pathlib.Path(__file__).with_name('prepare_tick_logic_archive.py')
    spec = importlib.util.spec_from_file_location('plot_nbt_reader', source)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def positions(bounds):
    lo, hi = bounds
    return {'min': lo, 'max': hi, 'size': [hi[i] - lo[i] + 1 for i in range(3)]}


def inventory_observation(be):
    items = be.get('Items')
    result = {'pos': [be.get(a) for a in 'xyz'], 'id': be.get('id'),
              'keys': sorted(be), 'items_tag': 'absent' if 'Items' not in be else
              'list' if isinstance(items, list) else 'other', 'occupied_slots': 0,
              'item_count': 0, 'item_ids': {}, 'nested_component_keys': [],
              'nested_container_entries': 0}
    ids, components = collections.Counter(), collections.Counter()
    nested = 0
    def walk(value):
        nonlocal nested
        if isinstance(value, dict):
            comp = value.get('components')
            if isinstance(comp, dict):
                components.update(comp.keys())
                content = comp.get('minecraft:container')
                if isinstance(content, list):
                    nested += len(content)
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)
    if isinstance(items, list):
        for item in items:
            if isinstance(item, dict):
                amount = item.get('count', item.get('Count', 0))
                if isinstance(amount, int) and amount > 0:
                    result['occupied_slots'] += 1
                    result['item_count'] += amount
                    ids[item.get('id', '?')] += amount
    walk(be)
    result['item_ids'] = dict(ids)
    result['nested_component_keys'] = dict(components)
    result['nested_container_entries'] = nested
    for field in ['TransferCooldown', 'LootTable', 'LootTableSeed', 'disabled_slots', 'RecipesUsed']:
        if field in be:
            result[field] = be[field]
    return result


def survey(path):
    helper = nbt_helpers()
    data = path.read_bytes()
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as stream:
        raw = stream.read(MAX_NBT + 1)
    if len(raw) > MAX_NBT:
        raise ValueError('Decompressed NBT exceeds128MiB')
    _, root = helper.NBTReader(raw).root()
    regions = root['Regions']
    # This survey makes its one-region geometry assumption explicit rather
    # than silently inventing coordinates for a different future artifact.
    if len(regions) != 1:
        raise ValueError('This component survey requires exactly one saved region')
    region_name, region = next(iter(regions.items()))
    size = [abs(region['Size'][a]) for a in 'xyz']
    base = [region['Position'][a] + min(0, region['Size'][a] + 1) for a in 'xyz']
    sx, sy, sz = size
    volume = sx * sy * sz
    if volume > MAX_CELLS:
        raise ValueError('Region exceeds25million cells')
    palette = region['BlockStatePalette']
    names = [s['Name'] for s in palette]
    bits = max(2, (len(palette) - 1).bit_length())
    mask = (1 << bits) - 1
    words = [w & ((1 << 64) - 1) for w in region['BlockStates']]
    if len(words) < (volume * bits + 63) // 64:
        raise ValueError('Packed block array is too short')
    air = {i for i, name in enumerate(names) if name in
           ['minecraft:air', 'minecraft:cave_air', 'minecraft:void_air']}
    blocks, histogram = {}, collections.Counter()
    y_counts, bins, mechanism_bins = collections.Counter(), collections.Counter(), collections.Counter()
    face_rows = {key: [] for key in ['x_min', 'x_max', 'y_min', 'y_max', 'z_min', 'z_max']}
    all_face_counts = collections.Counter()
    controls = []
    stride = sx * sz
    def pos(index):
        return [base[0] + index % sx, base[1] + index // stride,
                base[2] + (index // sx) % sz]
    for index in range(volume):
        word, offset = divmod(index * bits, 64)
        value = words[word] >> offset
        if offset + bits > 64:
            value |= words[word + 1] << (64 - offset)
        state = value & mask
        if state >= len(palette):
            raise ValueError('Invalid palette index')
        if state in air:
            continue
        blocks[index] = state
        if len(blocks) > MAX_NONAIR:
            raise ValueError('Non-air survey cap exceeded')
        name = names[state]
        p = pos(index)
        histogram[name] += 1
        y_counts[p[1]] += 1
        bins[(p[0] // 4, p[2] // 4)] += 1
        if mechanism(name):mechanism_bins[(p[0] // 4, p[2] // 4)] += 1
        if control(name):
            controls.append({'pos': p, 'state': palette[state]})
        touched = []
        for axis, extent in enumerate(size):
            if p[axis] == base[axis]: touched.append('xyz'[axis] + '_min')
            if p[axis] == base[axis] + extent - 1: touched.append('xyz'[axis] + '_max')
        for face in touched:
            all_face_counts[face] += 1
            if mechanism(name):
                face_rows[face].append({'pos': p, 'state': palette[state]})
    bes = region.get('TileEntities', [])
    entities = region.get('Entities', [])
    entity_inventory_rows = []
    passenger_types, entity_item_ids, entity_item_components = collections.Counter(), collections.Counter(), collections.Counter()
    def inspect_entity(value, root_entity=True):
        if isinstance(value, dict):
            if 'Pos' in value and 'id' in value:
                if not root_entity:passenger_types[value['id']] += 1
                if 'Items' in value:
                    row = inventory_observation(value)
                    row['pos'] = value['Pos']
                    entity_inventory_rows.append(row)
            if isinstance(value.get('count'), int) and isinstance(value.get('id'), str):
                entity_item_ids[value['id']] += value['count']
                if isinstance(value.get('components'), dict):entity_item_components.update(value['components'].keys())
            for child in value.values():inspect_entity(child, False)
        elif isinstance(value, list):
            for child in value:inspect_entity(child, False)
    for entity in entities:inspect_entity(entity)
    inventories = [inventory_observation(be) for be in bes if container(be.get('id', ''))]
    block_entity_stacks = []
    def inspect_be_stacks(value, be, path):
        if isinstance(value, dict):
            amount = value.get('count', value.get('Count'))
            if isinstance(amount, int) and amount > 0 and isinstance(value.get('id'), str):
                block_entity_stacks.append({'pos':[be.get(a) for a in 'xyz'], 'block_entity_id':be.get('id'),
                    'tag_path':path, 'item_id':value['id'], 'count':amount,
                    'component_keys':sorted(value.get('components',{}))})
            for key, child in value.items():inspect_be_stacks(child,be,path+'/'+key)
        elif isinstance(value,list):
            for i,child in enumerate(value):inspect_be_stacks(child,be,path+'/'+str(i))
    for be in bes:inspect_be_stacks(be,be,'')
    signs = []
    for be in bes:
        if 'sign' not in be.get('id', ''):
            continue
        text = ' | '.join(helper.text_components(t) for side in ['front_text', 'back_text']
                          for t in be.get(side, {}).get('messages', [])).strip(' |')
        if text:
            signs.append({'pos': [be.get(a) for a in 'xyz'], 'text': text})
    inventory_types = {}
    for kind in sorted({row['id'] for row in inventories}):
        rows = [row for row in inventories if row['id'] == kind]
        inventory_types[kind] = {'saved_block_entities': len(rows),
            'items_tag_absent': sum(row['items_tag'] == 'absent' for row in rows),
            'items_tag_empty_list': sum(row['items_tag'] == 'list' and row['occupied_slots'] == 0 for row in rows),
            'with_occupied_slots': sum(row['occupied_slots'] > 0 for row in rows),
            'occupied_slots': sum(row['occupied_slots'] for row in rows),
            'item_count': sum(row['item_count'] for row in rows)}
    # Exact six-neighbour physical components: all saved non-air states count,
    # including display floors/signs. Shared floors may merge many real builds.
    remaining = set(blocks)
    components = []
    while remaining:
        start = min(remaining) if len(components) < 1 else next(iter(remaining))
        remaining.remove(start)
        stack = [start]
        lo, hi = pos(start), pos(start)
        count = 0
        types = collections.Counter()
        component_controls = []
        while stack:
            i = stack.pop(); p = pos(i); name = names[blocks[i]]
            count += 1; types[name] += 1
            for a in range(3):
                lo[a] = min(lo[a], p[a]); hi[a] = max(hi[a], p[a])
            if control(name):component_controls.append({'pos': p, 'state': palette[blocks[i]]})
            x, y, z = i % sx, i // stride, (i // sx) % sz
            neighbours = []
            if x: neighbours.append(i - 1)
            if x + 1 < sx: neighbours.append(i + 1)
            if z: neighbours.append(i - sx)
            if z + 1 < sz: neighbours.append(i + sx)
            if y: neighbours.append(i - stride)
            if y + 1 < sy: neighbours.append(i + stride)
            for neighbour in neighbours:
                if neighbour in remaining:
                    remaining.remove(neighbour); stack.append(neighbour)
        if count >= 8:
            components.append({'bounds': positions((lo, hi)), 'non_air_blocks': count,
                'piston_blocks': sum(v for k, v in types.items() if 'piston' in k),
                'observer_blocks': types['minecraft:observer'],
                'control_positions': sorted(component_controls, key=lambda r: r['pos']),
                'container_blocks': sum(v for k, v in types.items() if container(k)),
                'block_counts': dict(sorted(types.items()))})
    components.sort(key=lambda c: (-c['non_air_blocks'], c['bounds']['min']))
    for i, c in enumerate(components):c['component_id'] = f'physical-{i:04d}'
    def inside(p, c, padding=0):
        return len(p) == 3 and all(isinstance(p[i], (int, float)) and
            c['bounds']['min'][i] - padding <= p[i] <= c['bounds']['max'][i] + padding for i in range(3))
    def distances(c):
        lo, hi = c['bounds']['min'], c['bounds']['max']
        return {axis + suffix: value for i, axis in enumerate('xyz') for suffix, value in
                [('_min', lo[i] - base[i]), ('_max', base[i] + size[i] - 1 - hi[i])]}
    for c in components:
        c['selection_face_distances'] = distances(c)
        c['touching_selection_faces'] = [f for f,d in distances(c).items() if d == 0]
        c['possibly_truncated'] = bool(c['touching_selection_faces'])
    candidates = []
    eligible = [c for c in components if 16 <= c['non_air_blocks'] <= 4000 and
                c['piston_blocks'] >= 2 and math.prod(c['bounds']['size']) <= 100000]
    # Sign positions are evidence anchors only. Bounds remain the exact physical
    # component bounds, not invented fixed extraction boxes around those signs.
    # This pin-specific shortlist deliberately retains two missing-control cases.
    anchors = [
        ([160,11,210], 'Near LucasDaRedstone 3x3 glass hipster label'),
        ([238,62,1], 'Near 4x4 full-lamp record label'),
        ([170,0,276], 'Near semi-seamless 3x3 label'),
        ([152,0,192], 'Near seamless hipster category sign'),
        ([123,54,149], 'Component beside SoulBanished repair note'),
        ([136,77,85], 'Near SpaceWalker / g4me4u label'),
    ]
    selected, annotations = [], {}
    def box_distance(p, c):
        return sum(max(c['bounds']['min'][i]-p[i], 0, p[i]-c['bounds']['max'][i]) for i in range(3))
    for p, label in anchors:
        options = [c for c in eligible if c['non_air_blocks'] >= 30 and
                   min(distances(c).values()) > 0 and box_distance(p,c) <= 8]
        if options:
            c = min(options, key=lambda c:(box_distance(p,c),c['non_air_blocks'],c['bounds']['min']))
            if c not in selected:
                selected.append(c)
                annotations[c['component_id']] = {'label':label, 'sign_anchor':p,
                    'sign_box_manhattan_distance':box_distance(p,c),
                    'selection_reason':'Nearby saved sign; association and whole-door identity remain unverified.'}
    # Fill remaining slots across the four visible vertical display bands,
    # preferring manageable controlled components without container dependence.
    bands = {}
    for c in eligible:
        if (c in selected or c['non_air_blocks'] < 40 or c['non_air_blocks'] > 1200 or
            min(distances(c).values()) < 2 or not c['control_positions'] or
            any(inside(e.get('Pos', []),c,2) for e in entities)):
            continue
        nearby_text = ' '.join(s['text'].lower() for s in signs if inside(s['pos'],c,3))
        if any(word in nearby_text for word in ['broken', 'uses mobs', 'tnt', 'egg timer']):continue
        bands.setdefault(c['bounds']['min'][1] // 51, []).append(c)
    for rows in bands.values():rows.sort(key=lambda c:(c['container_blocks'] > 0,c['non_air_blocks'],c['bounds']['min']))
    while len(selected) < 20 and any(bands.values()):
        for band in sorted(bands):
            if bands[band] and len(selected) < 20:
                c = bands[band].pop(0); selected.append(c)
                annotations[c['component_id']] = {'label':f'Controlled component in display band {band+1}',
                    'selection_reason':'Interior controlled physical component; stratified by height, simple/container-free first.'}
    for c in selected:
        distance = distances(c)
        inv = [row for row in inventories if inside(row['pos'], c)]
        labels = [row for row in signs if inside(row['pos'], c, 3)]
        ent = [e for e in entities if inside(e.get('Pos', []), c, 2)]
        touch = [face for face, dist in distance.items() if dist == 0]
        warning = ['Physical connectivity and nearby labels are candidate hypotheses, not verified individual doors.',
                   'Audit a dependency/travel halo and adjacent components before choosing any extraction box; do not crop by this box automatically.']
        if touch:warning.append('possibly_truncated: candidate touches the approximate source selection; do not interpret failure as an engine defect.')
        if not c['control_positions']:warning.append('No saved lever/button in this component; activation port must be established from source/reference.')
        if c['container_blocks']:warning.append('Container contents may be intentionally empty, absent or insufficient; no original-world evidence establishes missing data or its cause.')
        if ent:warning.append('Nearby authored entities require a broader oracle; current community capture does not load/capture them.')
        if any(any(word in row['text'].lower() for word in ['broken', 'mob', 'cart', 'cooldown', 'patient', 'wait']) for row in labels):
            warning.append('Nearby signs mention timing, entities or breakage; association is not certain, inspect those notes before activation.')
        candidates.append({'id': f'candidate-{len(candidates)+1:02d}', 'component_id': c['component_id'],
            **annotations[c['component_id']],
            'readiness': 'unvalidated', 'evidence_basis': 'static inspection',
            'classification': 'physical component candidate, door identity unconfirmed',
            'bounds': c['bounds'], 'selection_face_distances': distance,
            'possibly_truncated': bool(touch), 'touching_selection_faces': touch,
            'non_air_blocks': c['non_air_blocks'], 'piston_blocks': c['piston_blocks'],
            'observer_blocks': c['observer_blocks'], 'controls': c['control_positions'],
            'container_blocks': c['container_blocks'], 'saved_container_entities': inv,
            'nearby_signs': labels, 'nearby_authored_entity_count': len(ent),
            'warnings': warning,
            'reference_requirements': ['Confirm intended mechanism, controls and required initial contents from a known working source.',
                'Keep original version4671; capture exact raw placement, initial states and real activation with all dependent blocks.',
                'Use full open/close/reset cycles and a no-activation control; assert opening aperture and restoration, not just activity.',
                'Capture inventories/entities/moving-BE state where mechanism requires it; establish sufficient tick and travel bounds.']})
    entry = {'source': 'user-world-record-plot', 'path': 'WorldRecordPlot.litematic',
        'name': 'User WorldRecordPlot survey', 'sha256': hashlib.sha256(data).hexdigest(),
        'bytes': len(data), 'readiness': 'unvalidated', 'evidence_basis': 'static inspection',
        'priority': 'high', 'tags': ['user-provided', 'door-survey', 'approximate-selection'],
        'native_data_version': root.get('MinecraftDataVersion'),
        'litematic_version': root.get('Version'), 'litematic_subversion': root.get('SubVersion'),
        'decompressed_bytes': len(raw), 'saved_metadata': root.get('Metadata'),
        'region': {'name': region_name, 'saved_position': region['Position'],
            'saved_size': region['Size'], 'bounds': positions((base, [base[i]+size[i]-1 for i in range(3)])),
            'volume': volume, 'non_air_blocks': len(blocks), 'palette_states': len(palette),
            'pending_block_ticks': len(region.get('PendingBlockTicks', [])),
            'pending_fluid_ticks': len(region.get('PendingFluidTicks', []))},
        'block_counts': dict(sorted(histogram.items())),
        'counts': {'piston_blocks': sum(v for k,v in histogram.items() if 'piston' in k),
            'observers': histogram['minecraft:observer'], 'levers': histogram['minecraft:lever'],
            'buttons': sum(v for k,v in histogram.items() if k.endswith('_button')),
            'container_blocks': sum(v for k,v in histogram.items() if container(k)),
            'block_entities': len(bes), 'authored_entities': len(entities)},
        'block_entity_types': dict(sorted(collections.Counter(be.get('id', '?') for be in bes).items())),
        'entity_types': dict(sorted(collections.Counter(e.get('id', '?') for e in entities).items())),
        'nested_passenger_entity_types': dict(sorted(passenger_types.items())),
        'entity_item_observations': {'inventory_records':entity_inventory_rows,
            'retained_item_stack_counts':dict(sorted(entity_item_ids.items())),
            'retained_item_component_keys':dict(sorted(entity_item_components.items())),
            'scope':'Recursive saved entity stacks, including displayed/equipped items; distinct from block-container inventory.'},
        'inventory_summary': {'by_type': inventory_types,
            'all_block_entity_retained_stacks':block_entity_stacks,
            'containers_with_occupied_slots': sum(row['occupied_slots'] > 0 for row in inventories),
            'occupied_slots': sum(row['occupied_slots'] for row in inventories),
            'item_count': sum(row['item_count'] for row in inventories),
            'containers_with_nested_components': sum(bool(row['nested_component_keys']) for row in inventories),
            'nested_container_entries': sum(row['nested_container_entries'] for row in inventories),
            'occupied_or_special_records': [row for row in inventories if row['occupied_slots'] or row['nested_component_keys'] or 'LootTable' in row],
            'ambiguity': 'Observed absent/empty Items tags describe this file only. They do not establish original contents, loss, or Litematica causation.'},
        'selection_boundary_audit': {'scope': 'Single region equals entire source selection; inclusive exact face contact.',
            'all_non_air_face_counts': {f:all_face_counts[f] for f in face_rows},
            'mechanism_control_container_face_counts': {f:len(rows) for f,rows in face_rows.items()},
            'mechanism_control_container_face_positions': face_rows,
            'interpretation': 'Source selection was approximate. Face contacts may be cut mechanisms; mark possibly_truncated, never engine_fail from this evidence.'},
        'projection': {'kind': 'saved non-air count summed over all y, not ground occupancy', 'xz_bin_size': 4,
            'xz_bins': [[x,z,count] for (x,z),count in sorted(bins.items())],
            'mechanism_xz_bins': [[x,z,count] for (x,z),count in sorted(mechanism_bins.items())],
            'mechanism_scope': 'Controls, containers, piston-related states, observers, dust/power blocks, repeaters/comparators, rails, slime/honey, targets, sculk and copper bulbs; includes all y.',
            'y_non_air_counts': [[y,n] for y,n in sorted(y_counts.items())]},
        'component_method': 'Exact6-neighbour connectivity across all non-air blocks; retains signage/floors, may merge unrelated builds or split diagonal/QC dependencies.',
        'candidate_selection_method': 'Six sign-anchored bounded components, then controlled interior components stratified across four height bands; annotations are hypotheses, not verified door identities.',
        'physical_components_8plus': components,
        'candidates': candidates, 'control_positions': controls, 'signs': signs,
        'warnings': ['No simulation, functionality, parity or completeness is established.',
            'User reports possibly broken doors, missing controls/contents and outdated mechanics. These are investigation hypotheses.',
            'Native4671 save version does not prove designs were built for1.21.11.',
            'The approximate selection may truncate builds at any face; interior components can still have external dependencies.',
            'No source-world inventory comparison is available; do not attribute empty contents to Litematica.']}
    return {'schema_version': 1, 'curator': 'world-record-plot', 'entries': [entry],
        'shortlist': [{'source': entry['source'], 'path': entry['path'], 'candidate_id': c['id'],
                       'bounds': c['bounds'], 'readiness': 'unvalidated'} for c in candidates]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    if args.output.exists():parser.error('Use a new output path; source/evidence is not overwritten')
    result = survey(args.input)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False)+'\n')
    entry = result['entries'][0]
    print(json.dumps({'sha256':entry['sha256'], 'region':entry['region'], 'counts':entry['counts'],
        'inventory_summary':{k:v for k,v in entry['inventory_summary'].items() if k not in ['by_type','occupied_or_special_records']},
        'components':len(entry['physical_components_8plus']), 'candidates':len(entry['candidates']),
        'boundary':entry['selection_boundary_audit']['mechanism_control_container_face_counts']}, indent=2))


if __name__ == '__main__':main()
