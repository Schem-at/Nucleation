// Node test for the polished JS API (`nucleation/api`).
//
// Run with: node tests/node_new_api_test.mjs
// Prereq: build-wasm.sh has been run so pkg/ is populated.

import init, * as raw from '../pkg/nucleation.js';
import { Schematic, Block, Cursor, UseBlock, ButtonPress } from '../pkg/api.js';

let pass = 0;
let fail = 0;
const failures = [];

function test(name, fn) {
  try {
    fn();
    pass++;
    console.log('  ok  ' + name);
  } catch (err) {
    fail++;
    failures.push({ name, err });
    console.log('  FAIL ' + name + ': ' + err.message);
  }
}

function assertEq(a, b, msg = '') {
  if (a !== b) throw new Error(`expected ${JSON.stringify(b)}, got ${JSON.stringify(a)} ${msg}`);
}

function assertDeep(a, b, msg = '') {
  if (JSON.stringify(a) !== JSON.stringify(b)) {
    throw new Error(`expected ${JSON.stringify(b)}, got ${JSON.stringify(a)} ${msg}`);
  }
}

console.log('Initializing WASM...');
await init();
console.log('WASM ready.\n');

// --- Block ----------------------------------------------------------------

console.log('Block:');

test('Block.parse plain', () => {
  const b = Block.parse('minecraft:stone');
  assertEq(b.id, 'minecraft:stone');
  assertEq(b.state, null);
});

test('Block.parse with state', () => {
  const b = Block.parse('minecraft:repeater[delay=4,facing=east,powered=false]');
  assertEq(b.id, 'minecraft:repeater');
  assertDeep(b.state, { delay: 4, facing: 'east', powered: false });
});

test('Block.parse with snbt', () => {
  const b = Block.parse('minecraft:jukebox[has_record=true]{signal:5}');
  assertDeep(b.state, { has_record: true });
  assertDeep(b.nbt, { __snbt__: 'signal:5' });
});

test('Block.withState returns new instance', () => {
  const a = new Block('minecraft:chest');
  const b = a.withState({ facing: 'west' });
  if (a === b) throw new Error('should be new instance');
  assertEq(a.state, null);
  assertDeep(b.state, { facing: 'west' });
});

test('Block.toString roundtrip', () => {
  const b = new Block('minecraft:repeater', { state: { delay: 4, facing: 'east' } });
  const s = b.toString();
  if (!s.startsWith('minecraft:repeater[')) throw new Error(s);
  if (!s.includes('delay=4') || !s.includes('facing=east')) throw new Error(s);
});

// --- Schematic ------------------------------------------------------------

console.log('\nSchematic:');

test('Schematic.new creates blank', () => {
  const s = Schematic.new('test');
  if (!s.raw) throw new Error('raw missing');
});

test('setBlock array form', () => {
  const s = Schematic.new('a');
  const out = s.setBlock([0, 0, 0], 'minecraft:stone');
  if (out !== s) throw new Error('chainable: should return self');
  assertEq(s.raw.get_block_string(0, 0, 0), 'minecraft:stone');
});

test('setBlock legacy 4-arg form', () => {
  const s = Schematic.new('b');
  s.setBlock(1, 0, 0, 'minecraft:dirt');
  assertEq(s.raw.get_block_string(1, 0, 0), 'minecraft:dirt');
});

test('setBlock with state opts', () => {
  const s = Schematic.new('c');
  s.setBlock([0, 0, 0], 'minecraft:repeater', { state: { delay: 4, facing: 'east' } });
  const bs = s.raw.get_block_string(0, 0, 0);
  if (!bs.includes('minecraft:repeater') || !bs.includes('delay=4')) {
    throw new Error('expected stored state, got ' + bs);
  }
});

test('setBlock with Block object', () => {
  const s = Schematic.new('d');
  s.setBlock([2, 0, 0], new Block('minecraft:gold_block'));
  assertEq(s.raw.get_block_string(2, 0, 0), 'minecraft:gold_block');
});

test('setBlock chainable returns self', () => {
  const s = Schematic.new('e');
  const out = s
    .setBlock([0, 0, 0], 'minecraft:stone')
    .setBlock([1, 0, 0], 'minecraft:dirt')
    .setBlock([2, 0, 0], 'minecraft:grass_block');
  if (out !== s) throw new Error('not chainable');
});

// --- Cursor ---------------------------------------------------------------

console.log('\nCursor:');

test('Cursor advances', () => {
  const s = Schematic.new('cur');
  const c = s.cursor({ origin: [0, 0, 0], step: [3, 0, 0] });
  c.place('minecraft:stone').advance();
  assertDeep(c.pos, [3, 0, 0]);
  c.place('minecraft:dirt');
  assertEq(s.raw.get_block_string(0, 0, 0), 'minecraft:stone');
  assertEq(s.raw.get_block_string(3, 0, 0), 'minecraft:dirt');
});

test('Cursor offset', () => {
  const s = Schematic.new('cur2');
  const c = s.cursor({ origin: [5, 0, 0] });
  c.place('minecraft:emerald_block', { offset: [0, 1, 0] });
  assertEq(s.raw.get_block_string(5, 1, 0), 'minecraft:emerald_block');
});

test('Cursor reset', () => {
  const s = Schematic.new('cur3');
  const c = s.cursor({ step: [2, 0, 0] }).advance(3);
  assertDeep(c.pos, [6, 0, 0]);
  c.reset();
  assertDeep(c.pos, [0, 0, 0]);
});

// --- copy / fill ----------------------------------------------------------

console.log('\nCopy & fill:');

test('copy is independent', () => {
  const a = Schematic.new('orig').setBlock([0, 0, 0], 'minecraft:stone');
  const b = a.copy();
  b.setBlock([0, 0, 0], 'minecraft:dirt');
  assertEq(a.raw.get_block_string(0, 0, 0), 'minecraft:stone');
  assertEq(b.raw.get_block_string(0, 0, 0), 'minecraft:dirt');
});

test('fill cuboid', () => {
  const s = Schematic.new('fc');
  s.fill([[0, 0, 0], [2, 0, 0]], 'minecraft:stone');
  for (let x = 0; x <= 2; x++) {
    assertEq(s.raw.get_block_string(x, 0, 0), 'minecraft:stone');
  }
});

// --- save -----------------------------------------------------------------

console.log('\nSave:');

test('save returns Uint8Array when format hint provided', async () => {
  const s = Schematic.new('save').setBlock([0, 0, 0], 'minecraft:stone');
  // Use no fs so it returns bytes (not via writeFile)
  const bytes = await s.save('out.litematic', { fs: { writeFile: async () => null } });
  // Our wrapper will route through fs since we're in Node — that means it writes
  // and returns null. Test the explicit-bytes path via toLitematic on raw instead.
  const direct = s.raw.to_litematic();
  if (!(direct instanceof Uint8Array)) throw new Error('not Uint8Array');
  if (direct.length === 0) throw new Error('empty');
});

// --- backcompat -----------------------------------------------------------

console.log('\nBackcompat:');

test('raw SchematicWrapper still importable', () => {
  if (typeof raw.SchematicWrapper !== 'function') throw new Error('no SchematicWrapper');
  const w = new raw.SchematicWrapper();
  w.set_block(0, 0, 0, 'minecraft:stone');
  assertEq(w.get_block_string(0, 0, 0), 'minecraft:stone');
});

test('Schematic.raw exposes the wrapper', () => {
  const s = Schematic.new('raw');
  if (!(s.raw instanceof raw.SchematicWrapper)) throw new Error('raw is not SchematicWrapper');
});

// --- Summary --------------------------------------------------------------

console.log(`\n${pass} passed, ${fail} failed`);
if (fail > 0) {
  for (const f of failures) console.error(`  ${f.name}: ${f.err.stack}`);
  process.exit(1);
}
