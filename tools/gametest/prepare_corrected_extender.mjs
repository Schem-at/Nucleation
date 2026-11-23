// Generate the external vanilla scenario from the user's hash-pinned original.
// No schematic or expanded fixture is committed to the repository.
// node tools/gametest/prepare_corrected_extender.mjs ENGINE SOURCE OUTPUT.json
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, basename } from 'node:path';
import { pathToFileURL } from 'node:url';
import assert from 'node:assert/strict';

const protocol = JSON.parse(await readFile(new URL('../../docs/performance/schematic-candidates/validation/extender-corrected-protocol.json', import.meta.url)));
const bytes = await readFile(process.argv[3]);
assert.equal(createHash('sha256').update(bytes).digest('hex'), protocol.source_sha256, 'Requires the corrected original');
const { Schematic, TickSimulation, TickSettleMode } = await import(pathToFileURL(resolve(process.argv[2], 'index.mjs')));
const schematic = Schematic.fromData(bytes);
assert.equal(schematic.mcVersion(), 5023, 'Preserve source DV5023');
assert.deepEqual(JSON.parse(schematic.boundingBoxJson()), [0,0,0,296,3,21]);
const sim = TickSimulation.fromSchematic(schematic, TickSettleMode.InWorld, 0,100,0,'');
const fixture = JSON.parse(sim.worldSnapshotJson());
assert.equal(fixture.length, 13090);
const scenario = {
  format_version:1, data_version:4671, source_mode:'exact_states_no_datafix',
  case:'extender_corrected_cycle', negative_control:false, size:[297,4,22], origin:[0,100,0],
  watch:{min:[-4,-1,-1],max:[300,4,23]}, rng_seed:0, capture_ticks:protocol.ticks,
  source:{sha256:protocol.source_sha256,data_version:schematic.mcVersion(),filename:basename(process.argv[3]),
    description:'Author-supplied corrected schematic; no local block edits.'},
  fixture, actions:protocol.actions, capture_stride:20,
};
await writeFile(process.argv[4], JSON.stringify(scenario)+'\n', {flag:'wx'});
console.log(JSON.stringify({case:scenario.case,blocks:fixture.length,source_sha256:scenario.source.sha256,output:process.argv[4]}));
