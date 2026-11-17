import assert from 'node:assert/strict';
import { VisualStates, MeshInvalidation, affectedChunks, finalChanges } from './src/mesh_invalidation.ts';
const vanilla = new VisualStates(() => '{"variants":{"":{"model":"test"}}}');
const note = n => `minecraft:note_block[instrument=harp,note=${n},powered=false]`;
assert.equal(vanilla.key(note(1)), vanilla.key(note(2)));
assert.equal(vanilla.key('minecraft:moving_piston[facing=east,type=normal]'), 'minecraft:air');
assert.equal(vanilla.key('minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]'),
  vanilla.key('minecraft:scaffolding[bottom=false,distance=6,waterlogged=false]'));
assert.notEqual(vanilla.key('minecraft:scaffolding[bottom=false,distance=0,waterlogged=false]'),
  vanilla.key('minecraft:scaffolding[bottom=true,distance=0,waterlogged=false]'));
assert.notEqual(vanilla.key('minecraft:redstone_wire[power=0]'), vanilla.key('minecraft:redstone_wire[power=15]'));
assert.notEqual(vanilla.key('minecraft:scaffolding[waterlogged=false]'), vanilla.key('minecraft:scaffolding[waterlogged=true]'));
const custom = new VisualStates(() => '{"variants":{"note=1":{"model":"a"},"note=2":{"model":"b"}}}');
assert.notEqual(custom.key(note(1)), custom.key(note(2)), 'respect custom resource-pack variants');
const multi = new VisualStates(() => '{"multipart":[{"when":{"OR":[{"AND":[{"note":"1"}]}]},"apply":{"model":"a"}}]}');
assert.notEqual(multi.key(note(1)), multi.key(note(2)), 'respect nested multipart conditions');
for (const definition of [() => { throw Error(); }, () => 'broken', () => '{}']) {
  const fallback = new VisualStates(definition);
  assert.notEqual(fallback.key(note(1)), fallback.key(note(2)), 'unknown definitions stay conservative');
}
assert.deepEqual(affectedChunks([5,5,5],16), ['0,0,0']);
assert.equal(affectedChunks([0,0,0],16).length,8);
assert.equal(affectedChunks([-1,-1,-1],16).length,8);
assert.deepEqual(affectedChunks([-5,-5,-5],16), ['-1,-1,-1']);
assert.deepEqual(finalChanges([{pos:[1,2,3],to:'a'},{pos:[1,2,3],to:'b'}]), [{pos:[1,2,3],to:'b'}]);
const q = new MeshInvalidation();
q.change('chunk','cell','a','b'); assert(q.dirty.has('chunk'));
q.change('chunk','cell','b','a'); assert.equal(q.dirty.size,0,'cancel pending reversal');
q.change('chunk','cell','a','b'); assert(q.take('chunk'),'start B snapshot');
q.change('chunk','cell','b','a'); assert(q.dirty.has('chunk'),'A must follow in-flight B');
q.change('chunk','cell','a','b'); assert.equal(q.dirty.size,0,'return to in-flight B needs no second snapshot');
q.change('chunk','one','a','b'); q.change('chunk','two','a','b');
q.change('chunk','one','b','a'); assert(q.dirty.has('chunk'),'other changed cell still needs work');
assert(q.take('chunk')); assert(!q.take('chunk'));
console.log('Mesh invalidation: state identity, pack overrides, seams, coalescing and async baselines passed');
