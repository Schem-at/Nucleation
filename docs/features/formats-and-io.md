# Formats and I/O

Load a schematic from any supported container, edit it through the normal
[`Schematic` construction API](basics.md), and write it in another format.
Filesystem helpers detect input by content and select output from the extension:

```python
from nucleation import Schematic

build = Schematic.load_from_file("build.litematic")  # input format auto-detected
build.save_to_file("build.schem")                    # Sponge output
```

For in-memory pipelines, use bytes instead of filesystem paths:

```python
build = Schematic.from_data(open("mystery.bin", "rb").read())
```

The rest of this guide covers supported containers, detection, conversion, and
measured round-trip fidelity. For coordinates, block placement, automatic
region growth, and block-state strings, start with [Basics](basics.md).

## Supported formats

| Format | Extension | Read | Write | Notes |
| --- | --- | :---: | :---: | --- |
| **Litematica** | `.litematic` | ✅ | ✅ | Multi-region native; the reference format |
| **Sponge Schematic** | `.schem` | ✅ | ✅ | The WorldEdit / community standard (v2–v3) |
| **Bedrock structure** | `.mcstructure` | ✅ | ✅ | Java ↔ Bedrock translated on the way through |
| **Snapshot** | `.nusn` | ✅ | ✅ | Nucleation's own uncompressed format |
| **Anvil world** | dir · `.zip` · `.mca` | ✅ | ✅ | Real world folders and region files |
| **Legacy MCEdit** | `.schematic` | ✅ | — | The pre-Flattening format; import only |

Worlds are covered in [Chunk iteration, streaming, and worlds](streaming-and-worlds.md);
the rest are single-schematic containers.

## Detection is by content, not extension

`load_from_file` and the byte-level readers sniff the **magic bytes**, so a
`.schem` that is really a litematic still loads, and the extension only decides
the *writer* on save. (The Anvil readers are tried last, since their headers are
the least distinctive.)

```python
Schematic.from_data(open("mystery.bin", "rb").read())   # format figured out from the bytes
```

## Round-trip fidelity — measured, not claimed

`examples/readme/formats-and-io/round-trip.rs` builds one schematic (blocks, block-state
properties, and a chest carrying NBT), writes it to every writable format, reads
each back, and compares a **content-exact fingerprint** to the original. An
identical hash is proof the round-trip lost nothing:

| Format | Bytes | Fingerprint | Meaning |
| --- | ---: | --- | --- |
| litematic | 485 | ✅ identical | bit-exact, NBT and all |
| snapshot | 70 202 | ✅ identical | bit-exact (uncompressed, hence large) |
| sponge `.schem` | 328 | ≈ equal | every block, state, and tile-entity value preserved; the reader adds an empty `components` tag (a 1.20.5+ data-component placeholder), which a content-exact hash notices but which loses nothing |
| mcstructure | 945 | ✕ translated | Bedrock is a *different game edition* — block ids and states are remapped through GeyserMC mappings, so this is a translation, not a round-trip |

The download beside every illustration comes from this same script:
[`round-trip.schem`](../../examples/readme/formats-and-io/round-trip.rs) and `.litematic` are
written next to the fingerprint table.

**Where data changes on purpose:** only the two cross-boundary cases.
`.mcstructure` translates to Bedrock (see
[Versions and translation](versions-and-translation.md)); legacy MCEdit
`.schematic` is import-only because the pre-Flattening numeric-id format cannot
represent modern block states. Everything else preserves content.

## The same loop, three languages

Rust — the native API, `Result`-returning:

```rust
use nucleation::formats::{litematic, schematic as sponge};

let mut s = litematic::from_litematic(&std::fs::read("in.litematic")?)?;
s.set_block_from_string(1, 3, 1, "minecraft:glowstone")?;
std::fs::write("out.schem", sponge::to_schematic(&s)?)?;
// Auto-detect by content: nucleation::formats::manager::get_manager()
//     .lock().unwrap().read(&bytes)?
```

Python — filesystem helpers, format from the extension:

```python
s = Schematic.load_from_file("in.litematic")
s.set_block(1, 3, 1, "minecraft:glowstone")
s.save_to_file("out.schem")
```

JavaScript — the WASM build has no filesystem, so it is bytes in, bytes out
(binary payloads cross as base64):

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const s = Schematic.fromData(readFileSync("in.litematic"));
s.setBlock(1, 3, 1, "minecraft:glowstone");
writeFileSync("out.schem", Buffer.from(s.toSchematicB64(), "base64"));
```

Every Python snippet in these docs has a fully runnable version with captured
output under [`docs/readme-snippets/`](../readme-snippets/).
