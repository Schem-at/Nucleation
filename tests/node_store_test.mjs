// Node test for the WASM Store + transparent-I/O bindings.
//
// Run with: node tests/node_store_test.mjs
// Prereq: ./build-wasm.sh has been run so pkg/ is populated.

import init, * as raw from '../pkg/nucleation.js';

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
    console.log('  FAIL ' + name + ': ' + (err && err.message ? err.message : err));
  }
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg || 'assertion failed');
}
function eq(a, b, msg) {
  if (a !== b) throw new Error((msg || 'eq') + `: expected ${JSON.stringify(b)}, got ${JSON.stringify(a)}`);
}

const enc = (s) => new TextEncoder().encode(s);
const dec = (u) => new TextDecoder().decode(u);

// A JS Map-backed handler object for Store.fromCallbacks.
function mapHandlers(m) {
  return {
    get: (k) => (m.has(k) ? m.get(k) : null),
    put: (k, data) => { m.set(k, new Uint8Array(data)); },
    has: (k) => m.has(k),
    delete: (k) => { m.delete(k); },
    list: (prefix) => [...m.keys()].filter((k) => k.startsWith(prefix)),
    health: () => {},
  };
}

await init();
console.log('WASM ready.\n');

console.log('Store:');

test('open() + roundtrip (mem://)', () => {
  const store = raw.StoreWrapper.open('mem://');
  assert(store.get('absent') === undefined, 'absent key is undefined');
  store.put('a/b', enc('hello'));
  eq(dec(store.get('a/b')), 'hello', 'get returns put value');
  eq(store.has('a/b'), true, 'exists true');
  const keys = store.list('a/');
  eq(keys.length, 1, 'one key listed');
  eq(keys[0], 'a/b');
  store.health(); // must not throw
  store.delete('a/b');
  assert(store.get('a/b') === undefined, 'deleted key gone');
  eq(store.has('a/b'), false, 'exists false after delete');
});

test('new Store(url) constructor matches open()', () => {
  const store = new raw.StoreWrapper('mem://');
  store.put('k', enc('v'));
  eq(dec(store.get('k')), 'v');
});

test('fromCallbacks (JS Map-backed)', () => {
  const m = new Map();
  const store = raw.StoreWrapper.fromCallbacks(mapHandlers(m));
  store.put('x/1', enc('one'));
  store.put('x/2', enc('two'));
  eq(dec(store.get('x/1')), 'one');
  eq(store.has('x/2'), true);
  eq(store.list('x/').length, 2);
  store.delete('x/1');
  assert(store.get('x/1') === undefined, 'deleted');
  // data really lives in the JS Map
  eq(m.size, 1, 'one entry left in the Map');
});

test('putIfAbsent (CAS)', () => {
  const store = raw.StoreWrapper.open('mem://');
  eq(store.putIfAbsent('k', enc('first')), true, 'first wins');
  eq(store.putIfAbsent('k', enc('second')), false, 'second rejected');
  eq(dec(store.get('k')), 'first', 'value unchanged');
});

test('listPaginated (keyset)', () => {
  const store = raw.StoreWrapper.open('mem://');
  for (let i = 0; i < 5; i++) store.put(`p/${i}`, enc('x'));
  let { keys, next } = store.listPaginated('p/', null, 2);
  eq(keys.length, 2, 'limit honoured');
  assert(next !== null, 'more pages remain');
  const seen = [...keys];
  while (next !== null) {
    const r = store.listPaginated('p/', next, 2);
    seen.push(...r.keys);
    next = r.next;
  }
  seen.sort();
  eq(seen.length, 5, 'covers every key');
});

test('transparent IO: saveToStore / openFromStore (mem://)', () => {
  const store = raw.StoreWrapper.open('mem://');
  const s = new raw.SchematicWrapper();
  s.set_blocks(new Int32Array([0, 0, 0]), 'minecraft:stone');
  s.saveToStore(store, 'build.schem');
  assert(store.list('').includes('build.schem'), 'key was stored');
  const loaded = raw.SchematicWrapper.openFromStore(store, 'build.schem');
  assert(loaded.get_volume() >= 1, 'roundtrip preserved blocks');
});

test('transparent IO over a fromCallbacks store', () => {
  const m = new Map();
  const store = raw.StoreWrapper.fromCallbacks(mapHandlers(m));
  const s = new raw.SchematicWrapper();
  s.set_blocks(new Int32Array([0, 0, 0]), 'minecraft:stone');
  s.saveToStore(store, 'nested/build.litematic');
  assert(m.size >= 1, 'bytes landed in the JS Map');
  const loaded = raw.SchematicWrapper.openFromStore(store, 'nested/build.litematic');
  assert(loaded.get_volume() >= 1, 'roundtrip through callback store');
});

console.log(`\n${pass} passed, ${fail} failed`);
if (fail > 0) {
  process.exit(1);
}
