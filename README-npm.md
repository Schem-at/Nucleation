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

### Storage

The `Store` class is pluggable byte storage — it decides *where bytes live*,
independent of the schematic format. Keys are `/`-delimited UTF-8 paths.

```ts
import { Store } from "nucleation";

const store = new Store("mem://"); // throws on error

store.put("schems/base.litematic", out); // (key, data: Uint8Array) -> void
const data = store.get("schems/base.litematic"); // -> Uint8Array | undefined
store.has("schems/base.litematic"); // -> boolean
store.list("schems/"); // -> string[] (keys with this prefix)
store.delete("schems/base.litematic"); // -> void (idempotent)
```

A schematic can read/write a `Store` directly — the format is inferred from the
key's extension. This is the way to use remote/host storage on the web; it works
with `mem://` and with `Store.fromCallbacks({ get, put, has, delete, list, health })`.

```ts
const schem = SchematicWrapper.openFromStore(store, "schems/base.litematic");
schem.saveToStore(store, "schems/copy.schem"); // format from extension
```

> **Web = `mem://` only.** On `wasm32` the in-memory backend is the only one
> available — the filesystem, S3, Redis, and Postgres backends are native-only
> and compiled out of the WASM build. Errors surface as thrown JS exceptions.

---

## Diff & Fingerprint

Canonically fingerprint a build, dedup near-duplicates, and structurally diff two
builds — all under an equivalence ruleset chosen by **preset name**:

- `"exact"` — material- and orientation-sensitive (identical blockstates only).
- `"shape"` — occupancy only; palette and orientation ignored.
- `"structural"` — functional shape, rotation- and material-agnostic.
- `"redstone_computational"` (alias `"redstone"`) — redstone-logic equivalence,
  rotation- and cosmetic-material-agnostic.
- `"redstone_survival"` — like `"redstone"`, keeping survival material constraints.

`diff()` also takes an optional overrides object with per-edit cost weights and a
symmetry group: `{ cost_add?, cost_delete?, cost_change?, cost_swap?, symmetry? }`.

```ts
import { SchematicWrapper } from "nucleation";

const a = new SchematicWrapper();
a.from_litematic(aBytes);
const b = new SchematicWrapper();
b.from_litematic(bBytes);

// 32-hex canonical hash (rotation/translation/palette-agnostic per preset).
console.log(a.fingerprint("structural"));

// Cheap exact dedup + fuzzy FFT shape distance (0.0 == same shape).
if (a.isDuplicateOf(b, "structural")) console.log("duplicate build");
console.log("footprint distance:", a.footprintDistance(b, "structural"));

// Dims + token histogram as JSON.
console.log(a.signature("structural"));

// Structural diff -> DiffWrapper. `preset` defaults to "exact".
const d = a.diff(b, "redstone");
console.log("distance:", d.distance);
// support = fraction of the larger build's cells that aligned (confidence,
// NOT a similarity %).
console.log("support:", d.support);

// Each delta as its own SchematicWrapper.
d.added(); d.removed(); d.changed(); d.swapped(); d.markers();

// Lossless JSON round-trips via DiffWrapper.fromJson; summaryJson() is compact.
import { DiffWrapper } from "nucleation";
const full = d.toJson();
const restored = DiffWrapper.fromJson(full);
console.log(d.summaryJson());
```

The glowing-overlay GLB requires a build compiled with the `meshing` feature
(the published WASM bundle includes it):

```ts
// `afterGlb` is the meshed "after" build (Uint8Array); returns a new GLB.
const glb = d.toOverlayGlb(afterGlb); // -> Uint8Array
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

MIT. See the [main repository](https://github.com/Schem-at/Nucleation)
for details.
