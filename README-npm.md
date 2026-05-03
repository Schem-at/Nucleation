# Nucleation for JavaScript / TypeScript

[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)

JavaScript / TypeScript bindings for **Nucleation**, a high-performance
Minecraft schematic engine. Read, edit, build, simulate, and mesh
`.schematic`, `.litematic`, `.nbt`, and `.mcstructure` files — in the
browser or in Node.

The core is written in Rust and compiled to WebAssembly via wasm-bindgen.
A single package works in modern browsers, bundlers, and Node 18+.

---

## Install

```bash
npm install nucleation
```

```bash
pnpm add nucleation
```

```bash
bun add nucleation
```

---

## Quick start

```ts
import init from "nucleation";
import { Schematic } from "nucleation/api";

await init(); // load the wasm module

const bytes = await fetch("example.litematic").then((r) => r.arrayBuffer());
const schem = Schematic.open(new Uint8Array(bytes), {
  hint: "example.litematic",
});

schem.setBlock([0, 0, 0], "minecraft:gold_block");
schem.setBlock([0, 1, 0], "minecraft:repeater", {
  state: { delay: 4, facing: "east" },
});

const out = await schem.save("out.schem"); // Uint8Array
```

### Hot loops

```ts
// One call into wasm — millions of placements/sec.
const positions = Array.from({ length: 1_000_000 }, (_, x) => [x, 0, 0]);
schem.setBlocks(positions, "minecraft:stone");

// Pre-resolve once, place by index.
const stone = schem.prepareBlock("minecraft:stone");
for (const [x, y, z] of positions) {
  schem.place(x, y, z, stone);
}
```

### Tile-entity helpers

```ts
import { chest, sign, text, Item } from "nucleation/api";

schem.setBlock([0, 0, 0], "minecraft:chest", {
  state: { facing: "north" },
  nbt: chest([
    Item("minecraft:diamond", { count: 64 }),
    Item("minecraft:elytra"),
  ]),
});

schem.setBlock([0, 1, 0], "minecraft:oak_sign", {
  nbt: sign([text("Loot", { color: "gold" }), "this way →"]),
});
```

---

## Documentation

Full reference, including simulation, meshing, and resource-pack rendering,
lives in the main repository:

- [Nucleation on GitHub](https://github.com/Schem-at/Nucleation)
- [JavaScript API reference](https://github.com/Schem-at/Nucleation/blob/master/docs/javascript/README.md)
- [Schematic Builder guide](https://github.com/Schem-at/Nucleation/blob/master/docs/guide/schematic-builder.md)

---

## Why Rust + WASM?

Nucleation's core is shared across Rust, WebAssembly/JS, Python, and C/PHP —
one engine, one set of behaviour, one set of tests. The npm package you
install is the same engine that powers the Python and Rust libraries; the
wasm boundary is the only thing between your JS and native-speed parsing
and meshing.

## License

AGPL-3.0-only. See the [main repository](https://github.com/Schem-at/Nucleation)
for details.
