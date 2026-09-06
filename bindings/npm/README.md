# nucleation

A high-performance Minecraft schematic engine, compiled to WebAssembly from Rust. Parse, edit,
diff, fingerprint, and generate schematics in Node or the browser.

This package ships generated ESM bindings (with TypeScript typings) plus the WASM binary. It
requires Node 18+ or a bundler; no native dependencies.

## Install

```bash
npm install nucleation
```

## Quick start

```javascript
import { Schematic } from "nucleation";

const schematic = Schematic.create("demo");
schematic.setBlock(1, 2, 3, "minecraft:stone");
console.log(schematic.getBlockName(1, 2, 3)); // "minecraft:stone"

// Binary payloads cross the WASM boundary base64-encoded
const b64 = schematic.toLitematicB64();
const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
const loaded = Schematic.fromLitematic(bytes);
```

## What is included

The published WASM contains the core feature set: schematic editing, all schematic formats,
world import (from zip) and export, the schematic builder, the procedural building tool,
definition regions, diff and fingerprinting, autostack, NBT helpers, SDF sampling, and the
in-memory store.

Redstone simulation and mesh generation also work on WASM but are not compiled into the
published binary. To use them, build a custom WASM from the
[repository](https://github.com/Schem-at/Nucleation):

```bash
cargo build --release --target wasm32-unknown-unknown --lib --features bridge,simulation,meshing
```

and point the package's `diplomat.config.mjs` at your `nucleation.wasm`. GPU rendering and
embedded scripting are native-only.

## Documentation

- [JavaScript API reference](https://github.com/Schem-at/Nucleation/blob/master/docs/javascript/README.md)
- [Feature guides](https://github.com/Schem-at/Nucleation/tree/master/docs/guides)

## License

MIT

### Renderer-only WebAssembly

`import { Schematic } from "nucleation/renderer"` loads the bridge-only WASM build
for schematic viewing and editing. Simulation, meshing and voxelization remain
available from the main `nucleation` entry point. Each entry owns a separate WASM
instance; import only the entry needed by a worker.

Use `renderRegionsJson()` for region bounds and palettes, then
`regionBlockIndices(name, start, count)` for owned `Uint32Array` windows of at most
65,536 cells. Indices use x-fastest, then z, then y order. Call `clearContents()`
when done to release schematic storage immediately.
