#!/usr/bin/env node
// Exercise the exact Atlas engine and existing JS glue, without a renderer.
// Usage: node tools/check_tick_scaffolding_wasm.mjs [engine-directory]
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

const engine = resolve(process.argv[2] ?? 'apps/sim-lab-wasm/public/engine');
const { TickSimulation, TickSettleMode } = await import(pathToFileURL(`${engine}/index.mjs`));
const results = [];
for (const name of ['note_trigger', 'supports', 'waterlogged']) {
  const capture = JSON.parse(await readFile(`crates/mc-tick/tests/corpus/scaffolding12111/${name}.json`, 'utf8'));
  const palette = capture.fixture.map(({ state }) => {
    const [Name, properties] = state.split('[');
    return properties ? { Name, Properties: Object.fromEntries(properties.slice(0, -1).split(',').map(p => p.split('='))) } : { Name };
  });
  const text = JSON.stringify({ DataVersion: capture.source.data_version, size: capture.size, palette,
    blocks: capture.fixture.map((b, state) => ({ pos: b.pos, state })), entities: [] });
  const sim = TickSimulation.fromSnbt(text, TickSettleMode.InWorld, ...capture.origin,
    capture.actions.flatMap(a => a.state ?? []).join(';'));
  sim.setRngSeed(0n);
  const normalize = rows => rows.map(row => JSON.stringify(row)).sort();
  let frames = 0;
  for (const expected of [capture.initial, ...capture.snapshots]) {
    if (expected.tick > 0) {
      for (const action of capture.actions.filter(a => a.tick === expected.tick)) {
        if (action.type === 'use_note_block' || action.type === 'pull_lever') sim.useBlock(...action.pos);
        else if (action.type === 'set_block') sim.placeBlock(...action.pos, action.state);
        else throw new Error(`Unsupported test action ${action.type}`);
      }
      sim.step();
    }
    const actual = JSON.parse(sim.worldSnapshotJson()).filter(b => b.pos.every((n, i) => n >= capture.watch.min[i] && n <= capture.watch.max[i]));
    assert.deepEqual(normalize(actual.map(b => [...b.pos, b.state])), normalize(expected.blocks), `${name}: tick ${expected.tick}`);
    frames++;
  }
  results.push({ case: name, matching_frames: frames, changes: JSON.parse(sim.changesJson()).length });
}
console.log(JSON.stringify({ engine, status: 'matches_vanilla_block_captures', results }));
