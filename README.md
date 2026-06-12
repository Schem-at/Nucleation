# Nucleation

**Nucleation** is a high-performance Minecraft schematic engine written in Rust with full support for **Rust**, **WebAssembly/JavaScript**, **Python**, and **FFI-based integrations** like **PHP** and **C**.

> Built for performance, portability, and parity across ecosystems.

---

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)

---

## Features

### Core

- **Multi-format support**: `.schematic`, `.litematic`, `.nbt`, `.mcstructure`, and more
- **World parsing**: import whole Minecraft worlds (Anvil `.mca` region files, zipped or on-disk world folders), optionally bounded to a coordinate box — and export schematics back out as playable worlds
- **Memory-safe Rust core** with zero-copy deserialization
- **Cross-platform**: Linux, macOS, Windows (x86_64 + ARM64)
- **Multi-language**: Rust, JavaScript/TypeScript (WASM), Python, C/FFI

### Schematic Building

- **SchematicBuilder**: Create schematics with ASCII art and Unicode characters
- **Compositional Design**: Build circuits hierarchically from smaller components
- **Unicode Palettes**: Visual circuit design with intuitive characters (`→`, `│`, `█`, etc.)
- **Template System**: Define circuits in human-readable text format
- **CLI Tool**: Build schematics from command line (`schematic-builder`)

### Circuit Simulation

- **Redstone simulation** via MCHPRS integration (optional `simulation` feature)
- **TypedCircuitExecutor**: High-level API with typed inputs/outputs (Bool, U32, Ascii, etc.)
- **CircuitBuilder**: Fluent API for streamlined executor creation
- **DefinitionRegion**: Advanced region manipulation with boolean ops, filtering, and connectivity analysis
- **Custom IO**: Inject and monitor signal strengths at specific positions
- **Execution Modes**: Fixed ticks, until condition, until stable, until change
- **State Management**: Stateless, stateful, or manual tick control

### 3D Mesh Generation

- **Resource pack loading** from ZIP files or raw bytes
- **GLB/glTF export** for standard 3D viewers and engines
- **USDZ export** for Apple AR Quick Look
- **Raw mesh export** for custom rendering pipelines (positions, normals, UVs, colors, indices)
- **Per-region and per-chunk meshing** for large schematics
- **Greedy meshing** to reduce triangle count
- **Occlusion culling** to skip fully hidden blocks
- **Ambient occlusion** with configurable intensity
- **Resource pack querying** — list/get/add blockstates, models, and textures

### Storage

- **Pluggable byte storage** — a `Store` moves opaque bytes by `/`-delimited key (get, put, exists, delete, list, health)
- **Multiple backends behind feature flags**: in-memory, filesystem, S3/MinIO, Redis, Postgres
- **One synchronous API** — networked backends wrap async SDKs behind an internal runtime, so calls stay blocking
- **Built for moving artifacts** — ship schematics, renders, and previews to S3 (and friends) with a single URL

### Developer Experience

- **Bracket notation** for blocks: `"minecraft:lever[facing=east,powered=false]"`
- **Feature parity** across all language bindings
- **Comprehensive documentation** in [`docs/`](docs/)
- Seamless integration with [Cubane](https://github.com/Nano112/cubane)

---

## Installation

### Rust

```bash
cargo add nucleation
```

### JavaScript / TypeScript (WASM)

```bash
npm install nucleation
```

### Python

```bash
pip install nucleation
```

### C / PHP / FFI

Download prebuilt `.so` / `.dylib` / `.dll` from [Releases](https://github.com/Schem-at/Nucleation/releases)
or build locally using:

```bash
./build-ffi.sh
```

---

## Quick Examples

### Loading and Saving Schematics

#### Rust

```rust
use nucleation::UniversalSchematic;

let bytes = std::fs::read("example.litematic")?;
let mut schematic = UniversalSchematic::new("my_schematic");
schematic.load_from_data(&bytes)?;
println!("{:?}", schematic.get_info());
```

#### JavaScript (WASM)

```ts
import init from "nucleation";
import { Schematic } from "nucleation/api";
await init();

const bytes = await fetch("example.litematic").then((r) => r.arrayBuffer());
const schem = Schematic.open(new Uint8Array(bytes), { hint: "example.litematic" });
schem.setBlock([0, 0, 0], "minecraft:gold_block");
await schem.save("out.schem");
```

#### Python

```python
from nucleation import Schematic, sign, text

# Three explicit constructors (legacy `Schematic("file.schem")` still works):
schem = Schematic.open("example.litematic")        # load
fresh = Schematic.new("my_schematic")              # blank
templ = Schematic.from_template("ab\ncd")          # ASCII template

# Polished set_block: tuple coords, structured state and NBT, chainable.
schem.set_block((0, 0, 0), "minecraft:repeater",
                state={"delay": 4, "facing": "east"})
schem.set_block((0, 1, 0), "minecraft:oak_sign",
                state={"rotation": 8},
                nbt=sign([text("Hello", color="gold"), "world"]))

# Format inferred from extension.
schem.save("out.litematic")
```

For hot loops placing many blocks, prefer the batch / fast-path APIs:

```python
# 30+ M placements/sec — one native call.
schem.set_blocks([(x, 0, 0) for x in range(1_000_000)], "minecraft:stone")

# 10+ M placements/sec — pre-resolve once, place by index.
stone = schem.prepare_block("minecraft:stone")
place = schem.place
for x, y, z in positions:
    place(x, y, z, stone)
```

### Parsing Worlds and Region Files (Anvil / `.mca`)

Whole worlds load into the same `UniversalSchematic` as any other format, so everything downstream (diffing, fingerprinting, meshing, re-export) just works. Three inputs are supported:

- a **single region file** (`r.0.0.mca`)
- a **zipped world folder** (reads `region/*.mca` plus `entities/*.mca` on 1.17+)
- a **world directory on disk** (native targets only, not WASM)

Every importer has a `_bounded` variant taking `min_x, min_y, min_z, max_x, max_y, max_z` block coordinates — use it to carve out just the area you care about instead of loading the entire world.

#### Rust

```rust
use nucleation::formats::world;

// Single .mca region file
let data = std::fs::read("r.0.0.mca")?;
let schematic = world::from_mca(&data)?;

// Zipped world folder, restricted to a bounding box
let zip = std::fs::read("my_world.zip")?;
let spawn_area = world::from_world_zip_bounded(&zip, -128, 0, -128, 128, 256, 128)?;

// World directory on disk (not available on WASM)
let world_dir = world::from_world_directory(std::path::Path::new("saves/my_world"))?;

// And back out: export a schematic as a playable world
let zip_bytes = world::to_world_zip(&schematic, None)?; // or Some(WorldExportOptions { .. })
```

#### JavaScript (WASM)

```ts
import init, { SchematicWrapper } from "nucleation";
await init();

const wrapper = new SchematicWrapper();

// Single .mca region file
wrapper.from_mca(new Uint8Array(mcaBytes));

// Zipped world folder (bounded variant shown)
wrapper.from_world_zip_bounded(new Uint8Array(zipBytes), -128, 0, -128, 128, 256, 128);

// Export as a zipped world; options are passed as a JSON string
const worldZip = wrapper.to_world_zip(JSON.stringify({ world_name: "Generated" }));
```

#### Python

```python
from nucleation import Schematic

schem = Schematic.new("world_import")

# Single .mca region file
schem.from_mca(open("r.0.0.mca", "rb").read())

# Zipped world folder, bounded to a box
schem.from_world_zip_bounded(open("my_world.zip", "rb").read(),
                             -128, 0, -128, 128, 256, 128)

# World directory on disk
schem.from_world_directory("saves/my_world")

# Export back to a playable world
zip_bytes = schem.to_world_zip()                 # bytes of a zipped world
schem.save_world("out_world")                    # write world folder to disk
files = schem.to_world()                         # dict[str, bytes] of world files
```

World export accepts an options object (JSON in WASM/Python) with fields like `world_name`, `game_mode`, `difficulty`, `spawn_position`, `data_version`, `version_name`, `void_world`, `offset`, `allow_commands`, and `day_time` — all optional with sensible defaults.

See [`examples/convert_world.rs`](examples/convert_world.rs) (world zip → litematic + GLB) and [`examples/inspect_world.rs`](examples/inspect_world.rs) for complete programs, and the per-language docs ([Rust](examples/rust.md), [WASM](examples/wasm.md), [Python](examples/python.md), [FFI](examples/ffi.md)) for the full API.

#### Streaming massive worlds (constant memory)

For worlds too large to load at once, the streaming API processes one chunk at a time — peak memory is O(one chunk) for directory/MCA sources, O(one region file) for zip:

```rust
use nucleation::formats::world_stream::{WorldSource, WorldSink, diff_worlds};

// Walk every chunk in a world directory; extract all sign block-entities
let src = WorldSource::open_dir("saves/my_world")?;
for result in src.chunks() {
    let view = result?;                   // per-chunk Err = corrupt chunk; stream continues
    for be in view.block_entities() {
        if be.id.contains("sign") {
            let schem = view.to_schematic();   // bridge to UniversalSchematic for a single chunk
            // … process schem …
        }
    }
}

// Replay: copy the BEFORE world, then apply each ChunkDiff to transform before→after.
// (Applying diffs to "saves/after" directly would double-apply already-present changes.)
std::fs::copy_dir_all("saves/before", "saves/replay")?;  // or use fs_extra / your own copy
let diffs = diff_worlds(
    &WorldSource::open_dir("saves/before")?,
    &WorldSource::open_dir("saves/after")?,
    "exact",
)?;
let air = BlockState::new("minecraft:air".to_string());
let mut sink = WorldSink::open_existing("saves/replay")?;
for cd in diffs? {
    let cd = cd?;
    // removed+added+changed are what replay applies; swapped/palette_swaps are analysis-only
    sink.patch_chunk(cd.cx, cd.cz, |view| {
        for (pos, _) in &cd.diff.removed  { view.set_block(pos.0, pos.1, pos.2, &air); }
        for (pos, b) in &cd.diff.added    { view.set_block(pos.0, pos.1, pos.2, b); }
        for (pos, _, b) in &cd.diff.changed { view.set_block(pos.0, pos.1, pos.2, b); }
    })?;
}
sink.finish()?;
```

Memory model: the streaming path never materialises more than one chunk (directory/MCA) or one region file (zip) at a time. For per-language streaming docs see [Rust §9](examples/rust.md#9--world--region-parsing-anvil--mca), [Python §8](examples/python.md#8-world--region-parsing-anvil--mca), [WASM §7](examples/wasm.md#7--world--region-parsing-anvil--mca), [FFI §26](examples/ffi.md#26----world--region-parsing-anvil--mca).

##### Generating worlds from scratch

`WorldChunkView::new(cx, cz)` creates an empty chunk that you fill with blocks,
then hand to a `WorldSink`. Sections are allocated on demand, so memory stays
constant no matter how large the world is.

```rust
use nucleation::formats::world_stream::{WorldChunkView, WorldSink};
use nucleation::formats::world::WorldExportOptions;
use nucleation::BlockState;

let options = WorldExportOptions {
    spawn_position: Some((0, 64, 0)),
    biome: "minecraft:plains".to_string(),  // default applied to sections with no biome data
    ..Default::default()
};
let mut sink = WorldSink::create("out_world", Some(options))?;

// Canonical region-major order is fastest; out-of-order works via read-merge.
for cz in -8..8_i32 {
    for cx in -8..8_i32 {
        let mut chunk = WorldChunkView::new(cx, cz);
        for bx in (cx * 16)..(cx * 16 + 16) {
            for bz in (cz * 16)..(cz * 16 + 16) {
                let h = 60 + ((bx + bz) % 8) as i32;
                chunk.set_block(bx, h, bz, &BlockState::new("minecraft:grass_block".to_string()));
                for by in 0..h {
                    chunk.set_block(bx, by, bz, &BlockState::new("minecraft:stone".to_string()));
                }
            }
        }
        chunk.set_biome("minecraft:plains");  // call AFTER set_block; sections are created lazily
        sink.write_chunk(chunk)?;
    }
}
sink.finish()?;  // writes level.dat; void_world: true by default (nothing else generates)
```

**Biomes:** sections with no biome data get the `WorldExportOptions::biome` default (`"minecraft:plains"` unless overridden). Override per-chunk with `chunk.set_biome("minecraft:desert")` — call it after placing blocks, since sections are allocated lazily. Biome data in chunks you read and re-stream is preserved verbatim; only freshly created sections without biome data receive the default. Chunk-level granularity only — sub-chunk 3D biome editing (4×4×4 cells) is future work. Use `chunk.biome_palette()` to inspect which biomes are present in a chunk.

**Limitations:** lighting is recalculated by Minecraft on first load. Spawn point, game rules, and other level settings come from `WorldExportOptions`. See per-language docs for [Rust §9](examples/rust.md#9--world--region-parsing-anvil--mca), [Python §8](examples/python.md#8-world--region-parsing-anvil--mca), [WASM §7](examples/wasm.md#7--world--region-parsing-anvil--mca), [FFI §26](examples/ffi.md#26----world--region-parsing-anvil--mca).

### Building Schematics with ASCII Art

```rust
use nucleation::SchematicBuilder;

// Use Unicode characters for visual circuit design!
let circuit = SchematicBuilder::new()
    .from_template(r#"
        # Base layer
        ccc
        ccc

        # Logic layer
        ─→─
        │█│
        "#)
    .build()?;

// Save as litematic
let bytes = nucleation::litematic::to_litematic(&circuit)?;
std::fs::write("circuit.litematic", bytes)?;
```

**Available in Rust, JavaScript, and Python!** See [SchematicBuilder Guide](docs/guide/schematic-builder.md).

### Compositional Circuit Design

```rust
// Build a basic gate
let and_gate = create_and_gate();

// Use it in a larger circuit
let half_adder = SchematicBuilder::new()
    .map_schematic('A', and_gate)  // Use entire schematic as palette entry!
    .map_schematic('X', xor_gate)
    .layers(&[&["AX"]])  // Place side-by-side
    .build()?;

// Stack multiple copies
let four_bit_adder = SchematicBuilder::new()
    .map_schematic('F', full_adder)
    .layers(&[&["FFFF"]])  // 4 full-adders in a row
    .build()?;
```

See [4-Bit Adder Example](docs/examples/4-bit-adder.md) for a complete hierarchical design.

### CLI Tool

```bash
# Build schematic from text template
cat circuit.txt | schematic-builder -o circuit.litematic

# From file
schematic-builder -i circuit.txt -o circuit.litematic

# Choose format
schematic-builder -i circuit.txt -o circuit.schem --format schem

# Export as mcstructure
schematic-builder -i circuit.txt -o circuit.mcstructure --format mcstructure
```

### Pluggable Storage

Open and save schematics transparently from local files **or** remote stores —
the same `open`/`save` calls, with the format inferred from the extension:

```rust
use nucleation::UniversalSchematic;

// transparent: a path, file://, or s3://bucket/key.schem
let schem = UniversalSchematic::open("s3://my-bucket/builds/adder.schem")?;
schem.save("adder.litematic", None)?;

// explicit store + key — works for ANY backend (redis/postgres too)
let store = nucleation::store::open("redis://localhost:6379")?;
let schem = UniversalSchematic::from_store(store.as_ref(), "builds/adder.schem")?;
schem.save_to_store(store.as_ref(), "out/adder.schem", None)?;
```

`redis://`/`postgres://` can't carry a key in one string (their URL path is the
DB), so use the explicit-store form for those. This transparent open/save is
available in every binding (Python `Schematic.open("s3://…")`, the FFI
`schematic_open`, WASM `openFromStore`, JVM/PHP `open`).

For raw byte storage (PNG renders, arbitrary artifacts) use the `Store` directly:

```rust
use nucleation::store::{self, Store};

let s = store::open("s3://my-bucket/previews")?; // Box<dyn Store>
s.put("build/abc.png", &png_bytes)?;
let bytes: Option<Vec<u8>> = s.get("build/abc.png")?;
let keys: Vec<String> = s.list("build/")?;
s.delete("build/abc.png")?;
s.health()?;
```

The backend is chosen entirely by the URL scheme: `mem://` (always available),
`file:///abs/path`, `s3://bucket/prefix`, `redis://host:6379/0`, and
`postgres://user:pass@host/db`. Each non-memory backend is gated behind a Cargo
feature — `store-fs` (on by default), `store-s3`, `store-redis`, `store-pg`, and
`store-callback` — so you only compile in the backends you use. S3/MinIO reads
credentials from the standard `AWS_*` environment variables (`AWS_ENDPOINT_URL` +
`AWS_S3_FORCE_PATH_STYLE=true` for MinIO), and the Postgres table name can be
overridden with `NUC_STORE_PG_TABLE` (default `nucleation_store`).

---

## Advanced Examples

### Setting Blocks with Properties

```js
const schematic = new SchematicWrapper();
schematic.set_block(
	0,
	1,
	0,
	"minecraft:lever[facing=east,powered=false,face=floor]"
);
schematic.set_block(
	5,
	1,
	0,
	"minecraft:redstone_wire[power=15,east=side,west=side]"
);
```

[More in `examples/rust.md`](examples/rust.md)

### Redstone Circuit Simulation

```js
const simWorld = schematic.create_simulation_world();
simWorld.on_use_block(0, 1, 0); // Toggle lever
simWorld.tick(2);
simWorld.flush();
const isLit = simWorld.is_lit(15, 1, 0); // Check if lamp is lit
```

### High-Level Typed Executor

```rust
use nucleation::{TypedCircuitExecutor, IoType, Value, ExecutionMode};

// Create executor with typed IO
let mut executor = TypedCircuitExecutor::new(world, inputs, outputs);

// Execute with typed values
let mut input_values = HashMap::new();
input_values.insert("a".to_string(), Value::Bool(true));
input_values.insert("b".to_string(), Value::Bool(true));

let result = executor.execute(
    input_values,
    ExecutionMode::FixedTicks { ticks: 100 }
)?;

// Get typed output
let output = result.outputs.get("result").unwrap();
assert_eq!(*output, Value::Bool(true));  // AND gate result
```

**Supported types**: `Bool`, `U8`, `U16`, `U32`, `I8`, `I16`, `I32`, `Float32`, `Ascii`, `Array`, `Matrix`, `Struct`

See [TypedCircuitExecutor Guide](docs/guide/typed-executor.md) for execution modes, state management, and more.

### 3D Mesh Generation

```rust
use nucleation::{UniversalSchematic, meshing::{MeshConfig, ResourcePackSource}};

// Load schematic and resource pack
let schematic = UniversalSchematic::from_litematic_bytes(&schem_data)?;
let pack = ResourcePackSource::from_file("resourcepack.zip")?;

// Configure meshing
let config = MeshConfig::new()
    .with_greedy_meshing(true)
    .with_cull_occluded_blocks(true);

// Generate GLB mesh
let result = schematic.to_mesh(&pack, &config)?;
std::fs::write("output.glb", &result.glb_data)?;

// Or USDZ for AR
let usdz = schematic.to_usdz(&pack, &config)?;

// Or raw mesh data for custom rendering
let raw = schematic.to_raw_mesh(&pack, &config)?;
println!("Vertices: {}, Triangles: {}", raw.vertex_count(), raw.triangle_count());
```

---

## Diff & Fingerprint

Nucleation can canonically *fingerprint* a build, dedup near-identical builds, and
compute a structural *diff* (edit distance) between two builds — all under a
configurable equivalence ruleset selected by a **preset name**.

### Presets

Every fingerprint/diff call takes a preset string that decides what "the same"
means:

| Preset | Equivalence |
|--------|-------------|
| `exact` | Material- and orientation-sensitive: only identical blockstates in the same orientation match. |
| `shape` | Occupancy only — any solid block counts, palette and orientation ignored. |
| `structural` | Functional shape under symmetry: rotation- and material-agnostic per the structural ruleset. |
| `redstone_computational` (alias `redstone`) | Redstone-logic equivalence: tokenizes components by function, rotation-agnostic, ignores cosmetic materials. |
| `redstone_survival` | Like `redstone`, but distinguishes survival-relevant material constraints. |

Presets are the baseline; you can **opt into overrides** — per-edit cost weights
(`add`/`delete`/`change`/`swap`) and the symmetry group — on the `diff` call.

### Fingerprint & dedup

```rust
use nucleation::UniversalSchematic;
use nucleation::fingerprint::{FingerprintSpec, fingerprint, signature, footprint_distance, is_duplicate};

let a = UniversalSchematic::from_litematic_bytes(&a_bytes)?;
let b = UniversalSchematic::from_litematic_bytes(&b_bytes)?;

let spec = FingerprintSpec::from_preset("structural").expect("known preset");

// 32-hex canonical hash — rotation/translation/palette-agnostic per the preset.
let hash: String = fingerprint(&a, &spec).to_hex();
println!("fingerprint = {hash}");

// Cheap exact-equivalence dedup (fingerprint(a) == fingerprint(b)).
if is_duplicate(&a, &b, &spec) {
    println!("a and b are duplicates under `structural`");
}

// Dims + token histogram as JSON (a coarse descriptor / pre-filter).
let sig_json: String = signature(&a, &spec).to_json();

// Fuzzy FFT footprint distance — 0.0 means identical occupancy shape.
let fuzzy: f32 = footprint_distance(&a, &b, &spec);
println!("footprint distance = {fuzzy}");
```

### Diff

```rust
use nucleation::diff::{DiffSpec, SpecOverrides, diff};

// Resolve a preset (optionally with cost/symmetry overrides) to a DiffSpec.
let spec = DiffSpec::resolve("redstone", &SpecOverrides::default()).expect("known preset");
let d = diff(&a, &b, &spec);

println!("edit distance = {}", d.distance);
// `support` = fraction of the larger build's cells that aligned
// (alignment confidence, NOT a similarity percentage).
println!("support = {:.3}", d.support);

// Each delta projected back to a schematic you can save or mesh.
let added = d.added();      // blocks present only in B
let removed = d.removed();  // blocks present only in A
let changed = d.changed();  // B's version at changed cells
let swapped = d.swapped();  // palette-only swaps
let markers = d.markers();  // colored overlay (lime/red/yellow/light-blue)

// Lossless JSON (round-trips via `Diff::from_json`) and a compact summary.
let full = d.to_json();
let summary = d.summary_json();
let restored = nucleation::diff::Diff::from_json(&full)?;
```

### Glowing overlay GLB *(requires the `meshing` feature)*

Given the "after" build already meshed to GLB, paint the diff on top of it as a
glowing overlay:

```rust
# #[cfg(feature = "meshing")]
# fn overlay(d: &nucleation::diff::Diff, after_glb: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
use nucleation::diff::OverlayOptions;

let glb: Vec<u8> = d.to_overlay_glb(after_glb, &OverlayOptions::default())?;
std::fs::write("diff_overlay.glb", &glb)?;
# Ok(())
# }
```

The same API is available from every binding — see
[Python](README-python.md#diff--fingerprint),
[JavaScript/WASM](README-npm.md#diff--fingerprint),
[JVM](nucleation-jvm/README.md#diff--fingerprint), and
[C/FFI](examples/ffi.md).

---

## Development

```bash
# Build the Rust core
cargo build --release

# Build with simulation support
cargo build --release --features simulation

# Build with meshing support
cargo build --release --features meshing

# Build WASM module (includes simulation)
./build-wasm.sh

# Build Python bindings locally
maturin develop --features python

# Build FFI libs
./build-ffi.sh

# Run tests
cargo test
cargo test --features simulation
cargo test --features meshing
./test-wasm.sh  # WASM tests with simulation

# Pre-push verification (recommended before pushing)
./pre-push.sh  # Runs all checks that CI runs
```

---

## Documentation

### 📖 Language-Specific Documentation

Choose your language for complete API reference and examples:

- **[Rust Documentation](docs/rust/)** - Complete Rust API reference
- **[JavaScript/TypeScript Documentation](docs/javascript/)** - WASM API for web and Node.js
- **[Python Documentation](docs/python/)** - Python bindings API
- **[C/FFI Documentation](examples/ffi.md)** - C-compatible FFI for PHP, Go, etc.

### 📚 Shared Guides

These guides apply to all languages:

- [SchematicBuilder Guide](docs/shared/guide/schematic-builder.md) - ASCII art and compositional design
- [TypedCircuitExecutor Guide](docs/shared/guide/typed-executor.md) - High-level circuit simulation
- [Circuit API Guide](docs/shared/guide/circuit-api.md) - CircuitBuilder and DefinitionRegion
- [Unicode Palette Reference](docs/shared/unicode-palette.md) - Visual circuit characters

### 🎯 Quick Links

- [Main Documentation Index](docs/) - Overview and comparison
- [Examples Directory](examples/) - Working code examples
- [World & Region Parsing](#parsing-worlds-and-region-files-anvil--mca) - Import `.mca` files and whole worlds

---

## License

Licensed under the **GNU AGPL-3.0-only**.
See [`LICENSE`](./LICENSE) for full terms.

Made by [@Nano112](https://github.com/Nano112)
