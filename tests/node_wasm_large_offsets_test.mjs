import assert from 'node:assert/strict';
import { test } from 'node:test';
import * as rt from '../bindings/js/diplomat-runtime.mjs';

// A real wasm32 buffer with only a few touched pages. Returning signed i32
// addresses reproduces the failure without allocating a multi-million-block scan.
const memory = new WebAssembly.Memory({ initial: 32769, maximum: 32772 });
let next = 0x80000020;
let growOnAlloc = false;
const freed = [];
const wasm = {
    memory,
    diplomat_alloc(size, align) {
        if (growOnAlloc) { memory.grow(1); growOnAlloc = false; }
        next = Math.ceil(next / align) * align;
        const ptr = next; next += size + 16;
        return ptr | 0;
    },
    diplomat_free(ptr, size, align) { freed.push([ptr >>> 0, size, align]); },
};

test('strings, slices, structs and results round-trip above 2 GiB', () => {
    const text = rt.DiplomatBuf.str8(wasm, 'Westminster — café');
    assert.ok(text.ptr > 0x7fffffff);
    assert.equal(rt.readString8(wasm, text.ptr | 0, text.size), 'Westminster — café');
    const wide = rt.DiplomatBuf.str16(wasm, '玻璃');
    assert.equal(rt.readString16(wasm, wide.ptr | 0, wide.size), '玻璃');
    for (const [type, values] of [['u8', [0, 255]], ['i32', [-384, 1164]], ['f64', [-1.5, 2.75]], ['u64', [2n, 9n]]]) {
        const slice = rt.DiplomatBuf.slice(wasm, values, type);
        const wrapper = rt.DiplomatBuf.sliceWrapper(wasm, slice);
        const result = new rt.DiplomatSlicePrimitive(wasm, wrapper.ptr | 0, type, []);
        assert.deepEqual(Array.from(result.getValue()), values);
        wrapper.free();
    }
    const result = new rt.DiplomatReceiveBuf(wasm, 8, 4, true);
    new Int32Array(memory.buffer, result.buffer, 1)[0] = -384;
    new Uint8Array(memory.buffer, result.buffer + 7, 1)[0] = 1;
    assert.equal(result.resultFlag, 1);
    assert.equal(rt.enumDiscriminant(wasm, result.buffer | 0), -384);
    new Uint32Array(memory.buffer, result.buffer, 1)[0] = text.ptr;
    assert.equal(rt.ptrRead(wasm, result.buffer | 0), text.ptr);
    result.free(); text.free(); wide.free();
    assert.ok(freed.length >= 7);
});

test('string list survives memory growth during inner allocations', () => {
    const alloc = wasm.diplomat_alloc;
    let calls = 0;
    wasm.diplomat_alloc = (size, align) => {
        if (++calls === 2) growOnAlloc = true;
        return alloc(size, align);
    };
    const list = rt.DiplomatBuf.strs(wasm, ['palace', 'clock tower'], 'string8');
    const pointers = new Uint32Array(memory.buffer, list.ptr, 4);
    assert.equal(rt.readString8(wasm, pointers[0], pointers[1]), 'palace');
    assert.equal(rt.readString8(wasm, pointers[2], pointers[3]), 'clock tower');
    list.free();
    wasm.diplomat_alloc = alloc;
});

test('write buffers and function-parameter allocations use unsigned addresses', () => {
    const data = rt.DiplomatBuf.str8(wasm, '588 × 384 × 1164');
    let destroyed = false;
    const writes = { ...wasm,
        diplomat_buffer_write_create: () => wasm.diplomat_alloc(16, 4),
        diplomat_buffer_write_get_bytes: () => data.ptr | 0,
        diplomat_buffer_write_len: () => data.size,
        diplomat_buffer_write_destroy: () => { destroyed = true; },
    };
    const write = new rt.DiplomatWriteBuf(writes);
    assert.ok(write.buffer > 0x7fffffff);
    assert.equal(write.readString8(), '588 × 384 × 1164');
    write.free(); assert.equal(destroyed, true);
    const arena = new rt.FunctionParamAllocator();
    arena.reserve(rt.internalConstructor, wasm, 16);
    assert.ok(arena.alloc(4) > 0x7fffffff);
    data.free();
});

test('a genuinely out-of-bounds address still throws instead of wrapping', () => {
    assert.throws(() => rt.resultFlag(wasm, (memory.buffer.byteLength - 1) | 0, 4), RangeError);
});
