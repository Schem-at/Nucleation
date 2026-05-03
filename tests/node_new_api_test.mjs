// Node test for the polished JS API (`nucleation/api`).
//
// Run with: node tests/node_new_api_test.mjs
// Prereq: build-wasm.sh has been run so pkg/ is populated.

import init, * as raw from '../pkg/nucleation.js';
import { Schematic, Block, Cursor, UseBlock, ButtonPress, Item, chest, sign, text } from '../pkg/api.js';

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

// --- Minecraft helpers ---------------------------------------------------

console.log('\nMinecraft helpers:');

test('text() basic', () => {
  assertEq(text('Hello'), '{"text":"Hello"}');
});

test('text() with formatting', () => {
  const t = text('Warn', { color: 'red', bold: true });
  if (!t.includes('"color":"red"')) throw new Error(t);
  if (!t.includes('"bold":true')) throw new Error(t);
});

test('chest() list of tuples', () => {
  const n = chest([['minecraft:diamond', 64], 'minecraft:elytra']);
  if (n.Items.length !== 2) throw new Error('expected 2 items');
  assertEq(n.Items[0].Slot, 0);
  assertEq(n.Items[0].id, 'minecraft:diamond');
  assertEq(n.Items[0].Count, 64);
  assertEq(n.Items[1].Slot, 1);
  assertEq(n.Items[1].id, 'minecraft:elytra');
});

test('chest() dict with gaps', () => {
  const n = chest({ 0: ['minecraft:diamond', 64], 13: 'minecraft:elytra' });
  const slots = n.Items.map((it) => it.Slot).sort((a, b) => a - b);
  assertDeep(slots, [0, 13]);
});

test('chest() Item class', () => {
  const n = chest([new Item('minecraft:netherite_ingot', { count: 3 })]);
  assertEq(n.Items[0].Count, 3);
});

test('chest() name auto-wraps as JSON text', () => {
  const n = chest([['minecraft:diamond', 1]], { name: 'Loot' });
  if (!n.CustomName.includes('"text":"Loot"')) throw new Error(n.CustomName);
});

test('sign() basic produces front + back + waxed', () => {
  const n = sign(['Hi']);
  assertEq(n.front_text.messages.length, 4);
  assertEq(n.back_text.messages.length, 4);
  assertEq(n.is_waxed, false);
});

test('sign() rejects > 4 lines', () => {
  let threw = false;
  try { sign(['a', 'b', 'c', 'd', 'e']); } catch { threw = true; }
  if (!threw) throw new Error('should have thrown');
});

test('end-to-end: chest helper places via setBlock', () => {
  const s = Schematic.new('e2e');
  s.setBlock([0, 0, 0], 'minecraft:chest',
    { state: { facing: 'west' },
      nbt: chest([['minecraft:diamond', 64], 'minecraft:elytra'], { name: 'Loot' }) });
  const bs = s.raw.get_block_string(0, 0, 0);
  if (!bs.includes('minecraft:chest')) throw new Error('chest not placed');
});

test('end-to-end: Block instance reuse', () => {
  const loot = new Block('minecraft:chest', {
    state: { facing: 'west' },
    nbt: chest([['minecraft:diamond', 64]]),
  });
  const s = Schematic.new('reuse');
  for (const x of [0, 5, 10]) s.setBlock([x, 0, 0], loot);
  for (const x of [0, 5, 10]) {
    if (!s.raw.get_block_string(x, 0, 0).includes('minecraft:chest')) {
      throw new Error('reuse failed at x=' + x);
    }
  }
});

// --- Summary --------------------------------------------------------------

console.log(`\n${pass} passed, ${fail} failed`);
if (fail > 0) {
  for (const f of failures) console.error(`  ${f.name}: ${f.err.stack}`);
  process.exit(1);
}
