#!/usr/bin/env node
// Replay a block-only vanilla capture through the exact Atlas WASM/glue.
// Usage: node tools/check_tick_block_capture_wasm.mjs ENGINE CAPTURE[.gz] [SOURCE]
// SOURCE selects the production schematic importer; otherwise use the fixture.
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { gunzipSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const engine = resolve(process.argv[2]);
const capturePath = resolve(process.argv[3]);
const raw = await readFile(capturePath);
const captureBytes = capturePath.endsWith('.gz') ? gunzipSync(raw) : raw;
const c = JSON.parse(captureBytes);
assert.equal(c.mc_version, '1.21.11');
assert.equal(c.settle, 'in_world');
assert.equal(c.snapshots.length, c.capture_ticks);
assert.ok(c.fixture.every(b => !b.nbt), 'Block-entity payloads need a richer comparison');
assert.ok(!c.entities?.length && !c.item_entities?.length, 'Entity equality is outside this checker');
const { Schematic, TickSimulation, TickSettleMode } = await import(pathToFileURL(`${engine}/index.mjs`));
const extra = c.actions.flatMap(a => a.state ?? []).join(';');
let sim;
if (process.argv[4]) {
  const bytes = await readFile(process.argv[4]);
  assert.equal(hash(bytes), c.source.sha256, 'Original source hash');
  const schematic = Schematic.fromData(bytes);
  assert.equal(schematic.mcVersion(), c.source.data_version, 'Preserve source data version');
  sim = TickSimulation.fromSchematic(schematic, TickSettleMode.InWorld, ...c.origin, extra);
} else {
  const palette = c.fixture.map(({ state }) => {
    const [Name, properties] = state.split('[');
    return properties ? { Name, Properties: Object.fromEntries(properties.slice(0, -1).split(',').map(p => p.split('='))) } : { Name };
  });
  sim = TickSimulation.fromSnbt(JSON.stringify({ DataVersion: c.source.data_version, size: c.size, palette,
    blocks: c.fixture.map((b, state) => ({ pos: b.pos, state })), entities: [] }),
    TickSettleMode.InWorld, ...c.origin, extra);
}
sim.setRngSeed(BigInt(c.rng_seed));
const normalize = rows => rows.map(row => JSON.stringify(row)).sort();
const expectedHash = createHash('sha256'), actualHash = createHash('sha256');
let frames = 0, changes = 0;
for (const expected of [c.initial, ...c.snapshots]) {
  assert.equal(expected.players, 0);
  if (expected.tick > 0) {
    for (const action of c.actions.filter(a => a.tick === expected.tick)) {
      if (action.type === 'use_note_block' || action.type === 'pull_lever') sim.useBlock(...action.pos);
      else if (action.type === 'set_block') sim.placeBlock(...action.pos, action.state);
      else throw new Error(`Unsupported test action ${action.type}`);
    }
    sim.step();
  }
  assert.equal(Number(sim.tickCount()), expected.tick);
  const actual = JSON.parse(sim.worldSnapshotJson()).filter(b =>
    b.pos.every((n, i) => n >= c.watch.min[i] && n <= c.watch.max[i]));
  const a = normalize(actual.map(b => [...b.pos, b.state])), b = normalize(expected.blocks);
  assert.deepEqual(a, b, `${c.case}: tick ${expected.tick}`);
  expectedHash.update(`${expected.tick}:${JSON.stringify(b)}\n`);
  actualHash.update(`${expected.tick}:${JSON.stringify(a)}\n`);
  changes += JSON.parse(sim.changesJson()).length;
  sim.clearChanges();
  frames++;
}
console.log(JSON.stringify({ case: c.case, status: 'matches_vanilla_blocks', matching_frames: frames,
  changes, source_data_version: c.source.data_version, reference_version: c.mc_version,
  source_sha256: c.source.sha256, capture_sha256: hash(captureBytes),
  engine_sha256: hash(await readFile(`${engine}/nucleation.wasm`)),
  expected_sha256: expectedHash.digest('hex'), actual_sha256: actualHash.digest('hex'),
  importer: process.argv[4] ? 'original_schematic' : 'capture_fixture',
  scope: 'Every tick-end block within watch, including air; no entity, inventory, moving-BE or intra-tick equality.' }));
