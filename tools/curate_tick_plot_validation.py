#!/usr/bin/env python3
"""Reproduce static NBT triage and explicit door-validation proposals.

This reads every retained geometric candidate. It neither simulates nor promotes
archive status. Candidates with a piston are not thereby functional doors.
Run: python3 tools/curate_tick_plot_validation.py --archive /path/to/archive
"""
import argparse
import collections
import gzip
import hashlib
import itertools
import json
from pathlib import Path

from inspect_tick_world_record_plot import container
from prepare_tick_logic_archive import NBTReader, MAX_NBT
from serve_tick_archive import cells

SOURCES = {'world-record-parts', 'world-record-floor-parts'}
PISTON_BASES = {'minecraft:piston', 'minecraft:sticky_piston'}
AIR = 'minecraft:air'
REPO = Path(__file__).resolve().parents[1]

# Apertures are source-global, inclusive, inspected against actual NBT and
# textured native renders. Proposed closed states are assertions to TEST, not
# observations of a successful simulation. No missing items are invented.
PICKS = [
    dict(file='part0465.litematic', role='primary', label='Compact vertical 3x3',
         control=[159, 59, 212], aperture=[[156, 55, 213], [158, 57, 213]],
         rationale='Small clear quartz aperture; nine piston bases; observer ordering and delay-3 repeater. Sign warns about input timing.'),
    dict(file='part0622.litematic', role='primary', label='Vertical 5x5, three-block depth',
         control=[180, 56, 128], aperture=[[174, 56, 130], [178, 60, 130]],
         clearance=[[174, 56, 129], [178, 60, 131]],
         rationale='Clear 5x5 quartz perimeter; comparator and water-cauldron analog source; slime and concrete powder require runtime entity monitoring.'),
    dict(file='part0117.litematic', role='primary', label='Horizontal 6x6 slime/honey hatch',
         control=[93, 54, 191], aperture=[[92, 55, 198], [97, 55, 203]],
         rationale='Clear horizontal aperture and parked quartz panel; mixed slime/honey translation, observer chains, wall and trapdoor notifications.'),
    dict(file='part0101.litematic', role='primary', label='Sign-labelled vertical 7x7',
         control=[92, 53, 179], aperture=[[87, 60, 176], [93, 66, 176]],
         rationale='Source sign says Smallest 7x7; quartz panel parked below the opening, mixed slime/honey, note blocks and trapdoors.'),
    dict(file='part0004-floorpart0119.litematic', role='primary_with_inventory_risk', label='Floor-derived vertical 2x2',
         control=[144, 55, 170], aperture=[[143, 53, 171], [144, 54, 171]],
         rationale='Clear four-cell aperture, four sticky pistons, actual lever. Two opposing empty droppers are retained as saved; do not infer missing contents or repair. Exercises restored platform context.'),
    dict(file='part0468.litematic', role='reserve', label='Deep vertical 2x2, long piston chains',
         control=[153, 106, 214], aperture=[[161, 104, 213], [162, 105, 213]],
         clearance=[[161, 104, 213], [162, 105, 214]],
         rationale='Small opening with 25 piston bases and long horizontal chains; quartz stairs, lamps and multiple repeater delays. Geometric inference needs vanilla confirmation.'),
    dict(file='part0074.litematic', role='reserve', label='Horizontal 7x7 slime/honey hatch',
         control=[78, 53, 191], aperture=[[75, 55, 199], [81, 55, 205]],
         rationale='Larger counterpart to the 6x6 horizontal mechanism; tests scaling of moving panels with a separate source and exact aperture.'),
    dict(file='part0369.litematic', role='button_reserve', label='Button-driven compact 2x2',
         control=[141, 154, 255], aperture=[[139, 155, 255], [140, 156, 255]],
         rationale='Actual ceiling stone button; quartz aperture with six piston bases. Button pulse versus persistent lever timing must be established, not emulated with an injected power block.'),
    dict(file='part0425.litematic', role='diagnostic_counterexample', label='Source-sign broken mechanism',
         control=[145, 17, 234], aperture=None,
         rationale='Source sign explicitly says broken now. Detector rail powered=true without an authored entity and update-order warning. Preserve as a counterexample; matching engines must not certify functionality.'),
    dict(file='part0004-floorpart0210.litematic', role='diagnostic_inventory_button', label='Floor-derived button/tripwire 3x3',
         control=[172, 56, 278], aperture=[[166, 53, 277], [168, 55, 277]],
         clearance=[[166, 53, 276], [168, 55, 278]],
         rationale='Clear aperture, button and tripwire controls, two empty droppers, one triggered=true. Exercises inventory and saved-phase risks; no invented items or lever replacement.'),
]


def box_positions(box):
    return [list(p) for p in itertools.product(*(range(a, b + 1) for a, b in zip(*box)))]


def read_candidate(archive, row):
    path = (archive / row['local_path']).resolve()
    if not path.is_relative_to(archive.resolve()):
        raise ValueError('archive path escapes root')
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != row['sha256']:
        raise ValueError(f'content hash changed: {path}')
    with gzip.open(path, 'rb') as stream:
        data = stream.read(MAX_NBT + 1)
    if len(data) > MAX_NBT:
        raise ValueError(f'NBT exceeds bound: {path}')
    root = NBTReader(data).root()[1]
    saved = dict(cells(root))
    regions = list(root['Regions'].values())
    bes, entities = {}, []
    for region in regions:
        base = [region['Position'][a] + min(0, region['Size'][a] + 1) for a in 'xyz']
        for be in region.get('TileEntities', []):
            p = [be[a] for a in 'xyz']
            bes[tuple(a + b for a, b in zip(base, p))] = be
        entities.extend(region.get('Entities', []))
    names = collections.Counter(s.split('[', 1)[0] for s in saved.values())
    controls = [{'position': list(p), 'state': s} for p, s in saved.items()
                if s.startswith('minecraft:lever[') or s.split('[', 1)[0].endswith('_button')]
    containers = []
    for p, state in saved.items():
        if not container(state.split('[', 1)[0]):
            continue
        be = bes.get(p)
        items = None if be is None else be.get('Items')
        tag = 'missing_block_entity' if be is None else ('absent' if items is None else 'nonempty' if items else 'empty')
        containers.append({'position': list(p), 'state': state, 'items': tag,
                           'occupied_slots': len(items) if isinstance(items, list) else None})
    signs = []
    for p, be in bes.items():
        if 'sign' not in be.get('id', be.get('Id', '')):
            continue
        signs.append({'position': list(p), 'front': be.get('front_text', {}).get('messages', []),
                      'back': be.get('back_text', {}).get('messages', [])})
    piston_bases = sum(names[n] for n in PISTON_BASES)
    nonplatform = row.get('nonplatform_non_air_blocks', len(saved))
    if nonplatform is None:
        nonplatform = len(saved)
    risk = []
    if row.get('cutoff_face_contacts'):
        risk.append('source_boundary_contact')
    if not controls:
        risk.append('no_lever_or_button')
    if any(x['items'] != 'nonempty' for x in containers):
        risk.append('empty_or_absent_inventory_observation')
    if entities:
        risk.append('authored_entities')
    if row['source'] == 'world-record-floor-parts':
        risk.append('restored_platform_and_neighbor_dependency')
    if row.get('pistons', {}).get('moving_without_block_entity', 0):
        risk.append('moving_piston_missing_block_entity')
    if piston_bases == 0:
        risk.append('no_piston_base_fragment_review')
    elif piston_bases == 1 or nonplatform <= 16:
        risk.append('small_or_single_piston_review')
    if any(n.endswith(('concrete_powder', ':sand', ':red_sand', ':gravel', ':anvil')) for n in names):
        risk.append('gravity_blocks_runtime_entity_watch')
    if any('rail' in n for n in names):
        risk.append('rail_or_entity_dependency_review')
    if any(n.startswith('minecraft:') and 'command_block' in n for n in names):
        risk.append('command_block_not_a_standalone_circuit')
    text = json.dumps(signs, ensure_ascii=False).lower()
    if any(word in text for word in ['broken', 'outdated', 'doesnt work', "doesn't work"]):
        risk.append('source_sign_warns_failure')
    if len(saved) != row['non_air_blocks']:
        raise ValueError(f'non-air count differs from extraction: {row["path"]}')
    if len(entities) != row['entities']:
        raise ValueError(f'entity count differs from extraction: {row["path"]}')
    summary = dict(source=row['source'], file=row['path'], sha256=row['sha256'],
                   local_path=row['local_path'], non_air_blocks=len(saved),
                   nonplatform_non_air_blocks=nonplatform, piston_bases=piston_bases,
                   piston_heads=names['minecraft:piston_head'], moving_pistons=names['minecraft:moving_piston'],
                   entities=len(entities), block_entities=len(bes), actual_data_version=root.get('MinecraftDataVersion'),
                   controls=controls, containers=containers, risks=risk,
                   bounds=row['bounds'], nonplatform_block_bounds=row['block_bounds'],
                   block_names=dict(sorted(names.items())), signs=signs,
                   engine_support='requires native preflight; block presence alone does not prove support',
                   functional_status='not established by static curation')
    return summary, saved


def adapter(pick, row, saved):
    control = next(c for c in row['controls'] if c['position'] == pick['control'])
    kind = 'lever' if control['state'].startswith('minecraft:lever[') else 'button'
    positions = box_positions(pick['aperture']) if pick['aperture'] else []
    if any(saved.get(tuple(p), AIR) != AIR for p in positions):
        raise ValueError(f'proposed initial open aperture is not air: {pick["file"]}')
    clearance = box_positions(pick.get('clearance', pick['aperture'])) if positions else []
    if any(saved.get(tuple(p), AIR) != AIR for p in clearance):
        raise ValueError(f'proposed initial clearance is not air: {pick["file"]}')
    result = {**pick, 'source': row['source'], 'sha256': row['sha256'], 'local_path': row['local_path'],
              'bounds': row['bounds'], 'control_state': control['state'], 'risks': row['risks'],
              'containers': row['containers'], 'source_signs': row['signs'], 'status': 'proposed_unvalidated',
              'coordinates': 'source-global; subtract exporter origin for rebased fixture coordinates',
              'input_adapter': {'name': 'door_open', 'type': 'boolean', 'actuator': kind,
                                'position': pick['control'], 'initial_logical_value': True if positions else None,
                                'implementation': 'Use actual interaction; preserve saved facing/attachment. Check desired state before changing a lever.',
                                'mapping': 'proposed open=true -> lever powered=false, open=false -> powered=true' if kind == 'lever' and positions else 'not established; do not guess persistent boolean mapping for a pulse input'},
              'output_adapter': {'name': 'aperture', 'plane_box': pick['aperture'], 'cells': positions,
                                 'open': {'all_cells': AIR, 'clearance_cells': clearance},
                                 'closed': {'all_cells': 'minecraft:quartz_block'},
                                 'initial_saved_plane': 'all air' if positions else 'not mapped',
                                 'evidence': 'geometry and saved NBT only; runtime assertions remain pending'},
              'reset': {'method': 'restore pristine source checkpoint including block entities, inventories, entities, queues and RNG seed 0',
                        'settle_ticks': 20, 'must_assert': ['initial open aperture if mapped', 'no silent repair or inserted items', 'saved control attachment/facing']},
              'unit_tests': [
                  {'name': 'idle', 'ticks': 660, 'inputs': [], 'assertions': ['mapped aperture remains open', 'report unsolicited motion, item/entity loss and pending activity']},
                  {'name': 'two_cycles', 'ticks': 660, 'interaction_ticks': [21, 181, 341, 501],
                   'desired_open_after_settle': [False, True, False, True], 'observation_ticks': [180, 340, 500, 660],
                   'assertions': ['exact aperture materials at each phase', 'repeat first cycle output on second cycle', 'no moving-piston residue at settled endpoints', 'compare native/cached/shadow tick-end traces with fresh vanilla']},
              ],
              'benchmark': {'enabled': False, 'enable_after': 'two-cycle functionality and vanilla comparison pass with supported semantics',
                            'workload': 'same reset and exact interactions as unit test; report elapsed and thread CPU time',
                            'exclude_from_timer': ['parse', 'fixture construction', 'checkpoint restore', 'oracle assertions', 'rendering']}}
    if kind == 'button':
        result['unit_tests'][1]['protocol_status'] = 'pulse characterization required before treating four presses as close/open commands'
    if not positions:
        result['output_adapter'] = {'name': 'diagnostic_trace', 'status': 'no independent functional aperture oracle; never count motion or parity as door success'}
        result['unit_tests'][1]['protocol_status'] = 'diagnostic capture only; source warns that this mechanism is broken'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=REPO / 'docs/performance/schematic-candidates/plot-validation')
    args = parser.parse_args()
    library_data = (args.archive / 'library.json').read_bytes()
    entries = json.loads(library_data)['entries']
    candidates = [r for r in entries if r['source'] in SOURCES and r['classification'] == 'door_candidate'
                  and 'infrastructure_review' not in r.get('review_tags', [])]
    if len(candidates) != 803:
        raise ValueError(f'expected preserved population of 803; got {len(candidates)}; review scope explicitly')
    selected = {p['file']: p for p in PICKS}
    summaries, adapters = [], {}
    for row in sorted(candidates, key=lambda r: (r['source'], r['path'])):
        summary, saved = read_candidate(args.archive, row)
        summaries.append(summary)
        if row['path'] in selected:
            adapters[row['path']] = adapter(selected[row['path']], summary, saved)
    assert len(adapters) == len(PICKS)
    risks = collections.Counter(risk for row in summaries for risk in row['risks'])
    strata = collections.Counter()
    priority = ['moving_piston_missing_block_entity', 'authored_entities', 'source_boundary_contact',
                'no_piston_base_fragment_review', 'small_or_single_piston_review', 'no_lever_or_button',
                'empty_or_absent_inventory_observation', 'restored_platform_and_neighbor_dependency']
    for row in summaries:
        row['primary_stratum'] = next((p for p in priority if p in row['risks']), 'remaining_manual_control_candidates')
        strata[row['primary_stratum']] += 1
    doc = {'schema': 1, 'scope': 'Static NBT review, not simulation results',
           'source_plot_sha256': '5476ac349cd4529d7a237828633b9e7a995d70b8937f70e4836d2f9f8db66c1e',
           'library_sha256': hashlib.sha256(library_data).hexdigest(),
           'population': len(summaries), 'excluded_infrastructure': ['part0004', 'part0005', 'part0006'],
           'source_counts': dict(collections.Counter(r['source'] for r in summaries)),
           'overlapping_risk_counts': dict(sorted(risks.items())), 'exclusive_primary_strata': dict(sorted(strata.items())),
           'selected': [adapters[p['file']] for p in PICKS], 'candidates': summaries,
           'limitations': ['Boundary contact is a risk flag, not proof of truncation.',
                           'Empty or absent Items is observed, not proof that extraction lost contents.',
                           'Single pistons can be complete tiny doors; small-mechanism flags are review queues, not automatic rejection.',
                           'No-piston-base candidates are head/moving fragments needing structural review, not confirmed doors.',
                           'Native block support and runtime entity generation require the independent preflight and oracle runners.',
                           'Initial geometry and matching engines cannot prove intended functionality; assert open/closed panel cells.']}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'selection.json').write_text(json.dumps(doc, indent=2, ensure_ascii=False) + '\n')
    lines = ['# World-record plot validation selection', '',
             'All 803 geometric candidates were reread from hash-verified Litematic NBT. This is a selection and risk report, not a simulation pass list. The 746 first-pass and 57 floor-derived candidates remain intact. Three canonical infrastructure clusters are excluded from this population.', '',
             '## Overlapping risk observations', '', '| Observation | Candidates |', '|---|---:|']
    lines += [f'| {k.replace("_", " ")} | {v} |' for k, v in sorted(risks.items())]
    lines += ['', 'The categories overlap. Empty inventories do not establish lost items; boundary contacts do not establish a cut-off door. Single-piston and tiny mechanisms need inspection and are not automatically rejected. No-piston-base candidates are separately flagged for fragment review.', '',
              '## Exclusive triage strata', '', '| First applicable stratum | Candidates |', '|---|---:|']
    lines += [f'| {k.replace("_", " ")} | {v} |' for k, v in sorted(strata.items())]
    lines += ['', '## Selected protocols', '', '| Source part | Role | Proposed independent output |', '|---|---|---|']
    for pick in PICKS:
        a = adapters[pick['file']]
        lines.append(f'| `{pick["file"]}` | {pick["role"].replace("_", " ")} | {pick["label"]} |')
    lines += ['', 'Each selected JSON record contains the source SHA-256, exact control position and saved facing, full aperture cell list, reset requirements, timing, unit-test proposals and a disabled benchmark specification. Coordinates are source-global: use the fixture exporter origin when translating into a test world.', '',
              'For lever candidates, the proposed adapter exposes `door_open: bool`. Restore a pristine checkpoint with RNG seed 0, settle 20 ticks, and verify the saved open aperture. Request closed/open/closed/open at ticks 21/181/341/501 and assert quartz/air/quartz/air at 180/340/500/660. Run an independent 660-tick idle capture. The 160-tick allowance is a test budget, not a measured latency. Measure actual settling time after successful captures.', '',
              'Button candidates first require pulse characterization. Do not replace a button with a lever or directly alter power. The source-sign broken mechanism is deliberately diagnostic and has no functional-success oracle. No candidate enters timed benchmarks until its phase assertions and fresh vanilla comparison pass.', '',
              'Each reset must restore inventories, block entities, entities, pending queues and RNG, not just visible blocks. Compare full watched tick-end states, comparator outputs and inventories, and report unsupported/runtime-entity limits. Checkpoint creation, restoration, rendering and assertions stay outside the benchmark timer.', '']
    for pick in PICKS:
        a = adapters[pick['file']]
        lines += [f'### {pick["label"]}: `{pick["file"]}`', '', pick['rationale'], '',
                  f'Content: `{a["sha256"]}`. Control: `{a["control"]}` / `{a["control_state"]}`.', '',
                  f'Proposed plane: `{pick["aperture"]}`. Initial plane verified against saved NBT: {len(a["output_adapter"].get("cells", []))} air cells. Runtime output remains unvalidated.', '']
    lines += ['## Reproduce', '', '```sh', 'python3 tools/curate_tick_plot_validation.py --archive /path/to/Nucleation-schematic-archive', '```', '',
              'The script changes only this selection report. Archive validation status and the original sources remain unchanged. Fresh simulation evidence is maintained by the separate batch and vanilla validation runners.', '']
    (args.output / 'selection.md').write_text('\n'.join(lines))
    print(json.dumps({'population': len(summaries), 'selected': len(adapters), 'risk_counts': dict(risks),
                      'exclusive_strata': dict(strata)}, indent=2))


if __name__ == '__main__':
    main()
