// Verify the served WASM engine against the retained native comparator capture.
// Usage: node apps/schematic-atlas/verify-engine.mjs ARCHIVE OUTPUT.json
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import * as engine from '../sim-lab-wasm/public/engine/index.mjs';

const [archive, output] = process.argv.slice(2);
if (!archive || !output) throw new Error('ARCHIVE OUTPUT.json required');
const capturePath = path.join(archive, 'validation/candidate-run/vanilla-comparator/capture.json');
const captureBytes = fs.readFileSync(capturePath), capture = JSON.parse(captureBytes);
const library = JSON.parse(fs.readFileSync(path.join(archive, 'library.json')));
const row = library.entries.find(r => r.sha256 === capture.source.sha256);
if (!row) throw new Error('Missing pinned original');
const bytes = fs.readFileSync(path.join(archive, row.local_path));
const sha = value => createHash('sha256').update(value).digest('hex');
if (sha(bytes) !== capture.source.sha256) throw new Error('Original content hash changed');
const schematic = engine.Schematic.fromData(new Uint8Array(bytes));
const sim = engine.TickSimulation.fromSchematic(schematic, engine.TickSettleMode.InWorld, ...capture.origin, '');
const probes = [];
for (let y = capture.watch.min[1]; y <= capture.watch.max[1]; y++)
  for (let z = capture.watch.min[2]; z <= capture.watch.max[2]; z++)
    for (let x = capture.watch.min[0]; x <= capture.watch.max[0]; x++) probes.push([x,y,z]);
const probeJson = JSON.stringify(probes);
let action = 0, checked = 0;
for (const frame of [capture.initial, ...capture.snapshots]) {
  if (frame.tick) {
    while (action < capture.actions.length && capture.actions[action].tick === frame.tick) {
      const input = capture.actions[action++];
      if (input.type !== 'pull_lever') throw new Error('Unexpected action');
      sim.useBlock(...input.pos);
    }
    sim.step();
  }
  if (Number(sim.tickCount()) !== frame.tick) throw new Error('Tick counter mismatch');
  const expected = new Map(frame.blocks.map(b => [b.slice(0,3).join(','), b[3]]));
  const actual = JSON.parse(sim.readProbes(probeJson));
  for (let i = 0; i < probes.length; i++) {
    if (actual[i] !== (expected.get(probes[i].join(',')) || 'minecraft:air'))
      throw new Error(`Block mismatch at tick ${frame.tick}, ${probes[i]}: ${actual[i]}`);
  }
  checked++;
}
const result = {status:'passed', target:'wasm32-unknown-unknown', source_sha256:sha(bytes),
  capture_sha256:sha(captureBytes), frames:checked, watched_cells:probes.length,
  block_state_assertions:checked*probes.length, actions:action,
  scope:'Every watched block state including air at initial and each tick-end; actual lever actions through JS bindings. Comparator outputs/inventory slots independently compared by native checker, not this script.',
  max_browser_volume:engine.TickSimulation.maxVolume(),
  wasm_sha256:sha(fs.readFileSync(new URL('../sim-lab-wasm/public/engine/nucleation.wasm',import.meta.url)))};
fs.writeFileSync(output, JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(result));
