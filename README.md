# Nucleation

Nucleation is a high-performance Minecraft schematic engine written in Rust, with generated
bindings for C, C++, JavaScript/TypeScript (WASM), Kotlin/JVM, Python, and PHP.

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)

## What it does

- **Schematic formats**: read and write `.litematic`, Sponge `.schem`, WorldEdit `.schematic`,
  Bedrock `.mcstructure`, structure `.nbt`, and a fast binary snapshot format (`.nusn`), with
  format auto-detection.
- **World import and export**: parse whole worlds (Anvil `.mca` region files, zipped or on-disk
  world folders, optionally bounded to a coordinate box) into a schematic, and export schematics
  back out as playable worlds. A streaming API processes worlds chunk by chunk in constant memory.
- **Cross-version conversion**: convert block, block-entity, item, and entity data between
  Minecraft data versions (a Rust port of PaperMC's DataConverter), with loss reports on lossy
  down-converts.
- **Schematic building**: a template system for building schematics from ASCII or Unicode layer
  art, a procedural building tool (spheres, cuboids, cylinders, bezier curves, and more, filled
  by solid, gradient, or shaded brushes), and SDF-based shape and terrain generation.
- **Redstone simulation**: tick circuits headlessly via MCHPRS, inject and read signals at
  arbitrary positions, and drive circuits through a typed executor with named, typed inputs and
  outputs (booleans, integers, floats, ASCII).
- **Meshing and rendering**: turn schematics into GLB/glTF or USDZ meshes using any resource
  pack, and render PNG previews on the GPU, headlessly.
- **Diffing and fingerprinting**: structural diffs between schematics (added, removed, changed,
  swapped views), translation-invariant fingerprints, signatures, and duplicate detection.
- **Auto-stack**: detect the repeating lattice in a build and re-stamp it to a new size, for
  example a 4-bit adder to 8-bit, or a 32x32 screen to 64x64.
- **Storage**: a pluggable byte store (in-memory, filesystem, and S3, Redis, or Postgres behind
  feature flags) for moving schematics and renders around with a single URI.
- **Embedded scripting**: generate schematics from Lua or JavaScript scripts.
- **Block database**: a vendored copy of blockpedia (`nucleation::blockpedia`) — Minecraft block
  facts with texture-derived colors, palette and gradient generation, block-state queries and
  transforms, and Java-Bedrock blockstate and block-entity translation via Geyser mappings.

## One API, seven languages

Since v0.3.0 every language binding is generated from a single annotated-Rust source of truth
(`src/bridge/`) using [Diplomat](https://github.com/rust-diplomat/diplomat). The bindings are
committed under `bindings/`, regenerated and diffed in CI so they can never go stale, and every
language exposes the same types and methods with per-language casing and idioms:

| Language | Package | Errors | Naming |
| --- | --- | --- | --- |
| Rust | `nucleation` crate (native API) | `Result` | `snake_case` |
| C | Release archive (`include/` + library) | result structs | `Schematic_set_block` |
| C++ | Release archive (header-only over C ABI) | `diplomat::result` | `set_block` |
| JavaScript | `npm install nucleation` | exceptions | `setBlock` |
| Kotlin/JVM | Release JAR (JNA) | `kotlin.Result` | `setBlock` |
| Python | `pip install nucleation` | exceptions | `set_block` |
| PHP | Release archive (`php/` + FFI) | `DiplomatError` | `setBlock` |

## Installation

```bash
# Rust
cargo add nucleation

# JavaScript / TypeScript (Node >= 18 or a bundler)
npm install nucleation

# Python (CPython 3.12+)
pip install nucleation
```

For C, C++, Kotlin, and PHP, download the platform archive or JAR from
[Releases](https://github.com/Schem-at/Nucleation/releases), or build locally:

```bash
cargo build --release --lib --features bridge        # core surface
cargo build --release --lib --features bridge-full   # + meshing, simulation, rendering, scripting
```

### What ships in the published packages

Published artifacts (npm, PyPI, release archives, JAR) contain the core feature set: schematics,
formats, world import/export and streaming, builder, building tool, definition regions,
diff/fingerprint, autostack, NBT helpers, SDF, and the in-memory/filesystem store.

Meshing, rendering, simulation, and scripting are compiled in when you build the native library
yourself with `--features bridge-full` (or any subset, for example `--features bridge,simulation`).
Simulation and meshing also work on WASM. See the per-language docs for details.

## Quick start

### Rust

```rust
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut schematic = UniversalSchematic::new("demo".to_string());
    schematic.set_block_from_string(0, 0, 0, "minecraft:stone")?;
    schematic.set_block_from_string(1, 0, 0, "minecraft:lever[face=floor,facing=east]")?;

    let bytes = nucleation::formats::litematic::to_litematic(&schematic)?;
    let loaded = nucleation::formats::litematic::from_litematic(&bytes)?;
    assert_eq!(
        loaded.get_block(0, 0, 0).map(|b| b.name.as_str()),
        Some("minecraft:stone")
    );
    Ok(())
}
```

### JavaScript

```javascript
import { Schematic } from "nucleation";

const schematic = Schematic.create("demo");
schematic.setBlock(1, 2, 3, "minecraft:stone");
console.log(schematic.getBlockName(1, 2, 3)); // "minecraft:stone"

// Serialize to litematic bytes (base64 across the WASM boundary)
const bytes = Uint8Array.from(atob(schematic.toLitematicB64()), (c) => c.charCodeAt(0));
const loaded = Schematic.fromLitematic(bytes);
```

### Python

```python
import nucleation

schematic = nucleation.Schematic.create("demo")
schematic.set_block(1, 2, 3, "minecraft:stone")
print(schematic.get_block_name(1, 2, 3))  # "minecraft:stone"

schematic.save_to_file("demo.litematic")
loaded = nucleation.Schematic.load_from_file("demo.litematic")
```

### Kotlin

```kotlin
import at.schem.nucleation.*

val schematic = Schematic.create("demo")
schematic.setBlock(1, 2, 3, "minecraft:stone").getOrThrow()
println(schematic.getBlockName(1, 2, 3).getOrThrow()) // "minecraft:stone"
```

### PHP

```php
<?php
require "php/index.php";

use Stencil\Lib;
use Stencil\Schematic;

Lib::init("/path/to/libnucleation.so");

$schematic = Schematic::create("demo");
$schematic->setBlock(1, 2, 3, "minecraft:stone");
echo $schematic->getBlockName(1, 2, 3); // "minecraft:stone"
```

### C

```c
#include "Schematic.h"
#include <string.h>

int main(void) {
    DiplomatStringView name = {"demo", 4};
    Schematic *s = Schematic_create(name);

    DiplomatStringView stone = {"minecraft:stone", 15};
    Schematic_set_block(s, 1, 2, 3, stone);

    char buf[256];
    DiplomatWrite w = diplomat_simple_write(buf, sizeof(buf));
    Schematic_get_block_name(s, 1, 2, 3, &w);

    Schematic_destroy(s);
    return 0;
}
```

## Documentation

Full documentation lives in [`docs/`](docs/):

- [Documentation index](docs/README.md)
- Per-language references: [Rust](docs/rust/), [JavaScript](docs/javascript/),
  [Python](docs/python/), [Kotlin](docs/kotlin/), [PHP](docs/php/), [C](docs/c/),
  [C++](docs/cpp/)
- [Feature guides](docs/guides/) (concepts, data models, and JSON schemas shared by all
  bindings)

## Development

```bash
cargo test                                   # core test suite
cargo build --lib --features bridge          # build the bridge surface

# Regenerate the committed bindings from src/bridge/ (requires the diplomat-tool fork)
./tools/gen-bindings.sh

# End-to-end smoke tests for the generated bindings
./examples/bridge_smoke/c/run.sh
./examples/bridge_smoke/js/run.sh
./examples/bridge_smoke/python/run.sh
php -d ffi.enable=1 -r 'require "examples/bridge_smoke/php/main.php";'
```

CI regenerates the bindings and fails on any diff, checks coverage of the full pre-0.3.0 FFI
surface, and runs the smoke tests on every push.

### Minecraft block data

The block database (formerly the standalone `blockpedia` crate) lives in-tree at
`src/blockpedia/`. Its data ships as gzipped snapshots in `data/blockpedia/` — currently pinned
to Minecraft **26.2** (Java block states from Mojang's own data generator, official block
semantics — kind/base-block/tags/full-cube geometry, Bedrock block states, Geyser blockstate
mappings, and a color cache derived from the vanilla texture pack) — and `build.rs` bakes them
into static PHF tables at compile time. Normal builds never touch the network.

To refresh the Java data for a new Minecraft version (needs a JRE new enough for the server
jar on `PATH`; MC 26.x wants Java 25+):

```bash
# 1. Vanilla report converter: downloads the server jar, runs Mojang's data
#    generator (--reports), and rebuilds prismarinejs_blocks.json.gz — the
#    report is authoritative for the block list, properties and state ids;
#    enrichment fields (transparency, hardness, light, ...) carry forward
#    from the previous snapshot, and blocks new in the version are enriched
#    from an analogue block or a model-shape heuristic over the client jar
#    (the run prints the added/removed diff and every derived fact).
#    Also rebuilds block_semantics.json.gz from official data only:
#    - kind + base block: the report's definition.type / definition.base_state
#      (stairs), plus a model-texture linkage for the other shape variants
#      (oak_slab renders with block/oak_planks, owned by oak_planks)
#    - tags: every data/minecraft/tags/block/** tag from the server jar's
#      inner (bundler) jar, nested #tag refs resolved
#    - full_cube: blockstate models root in a cube-family template or carry
#      a full 16x16x16 element
#    These drive BlockFacts::{kind, base_block, has_tag, is_full_cube},
#    blocks_by_tag/variants_of, and the BlockFilter/only_solid classifiers
#    (which no longer guess from name substrings).
cargo run --release --bin refresh-block-data --features mc-data-refresh

# 2. Texture colors: downloads the client jar, extracts block textures,
#    regenerates color_cache.json.gz (alpha-weighted averages + biome tints).
cargo run --release --bin fetch-texture-colors --features mc-data-refresh
```

Both tools take the version as an optional trailing arg (`-- 26.2`) and default to the
manifest's latest release, so a routine bump needs no code edits. A normal `cargo build`
afterwards bakes the new tables in.

The PrismarineJS `blocks.json` schema is kept as the on-disk format (PrismarineJS itself has
no 26.x data). `tests/blockpedia_data_refresh.rs` guards data currency.

#### Java ↔ Bedrock mappings

- `geyser_mappings.json.gz` — regenerated from **GeyserMC/mappings** (`blocks.nbt` @
  [`efe0f2c`](https://github.com/GeyserMC/mappings/commit/efe0f2cabeaf4c8147c45e63f2d744e90d3b4156),
  "Mappings for Minecraft Java **26.2**"). GeyserMC retired the old `mappings-generator`
  JSON dumps; the canonical data is now gzipped NBT: a `bedrock_mappings` list with one
  compound per Java blockstate **in runtime state-id order** (java side implicit by index;
  `bedrock_identifier` absent ⇒ same name as Java, `state` absent ⇒ bedrock default state).
  `refresh-bedrock-mappings` converts that back into the JSON schema `build.rs` consumes,
  reconstructing the java side from `prismarinejs_blocks.json.gz` (state-id enumeration
  validated 32,366/32,366 against the vanilla 26.2 report). All 32,366 Java 26.2 states are
  mapped (was 29,671 at the 1.21.x pin; all carried-over entries identical, +2,695 gained,
  none lost), so the identity fallback for unmapped blocks is currently unused:

  ```bash
  cargo run --release --bin refresh-bedrock-mappings --features mc-data-refresh -- \
      --data-version 4903   # java world data version, for provenance only
  ```

- `bedrock_block_states.json.gz` — PrismarineJS `data/bedrock/1.26.30/blockStates.json`
  gzipped verbatim (content-identical to the previous snapshot; the per-state `version`
  field `1.21.60.33` is Bedrock's *state-format* version, which hasn't bumped since —
  the palette content is current and includes the 26.x cinnabar/sulfur blocks). Every
  `bedrock_identifier` emitted by the mappings exists in this palette.

## License

MIT. See [LICENSE](LICENSE).
