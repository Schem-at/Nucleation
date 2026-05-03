# Nucleation v0.2.3

Simulation surface gets a polished read API, the renderer learns to
aim at custom targets, and one nasty redstone bug we tracked into the
MCHPRS redpiler is now fixed at the source.

## What's new

### Direct lever control + idempotent simulate events

`simulate(events=...)` now accepts a third event type alongside
`UseBlock` and `ButtonPress`:

```python
from nucleation import LeverState

# Idempotent — calling twice with state=True does NOT toggle.
schem.simulate(events=[LeverState((0, 1, 3), state=True)], ticks=2)
schem.simulate(events=[LeverState((0, 1, 3), state=False)], ticks=2)
```

Backed by a new `MchprsWorld.set_lever_power(x, y, z, powered)`
pymethod that wires straight into the redpiler.

### Polished state accessors — read what's actually happening

The common questions about a simulated circuit have direct methods on
the schematic now, and they all read from the cached simulation world
(no schematic round-trip):

```python
schem.is_powered((0, 1, 0))      # bool — unified power check
schem.is_lit((2, 1, 0))          # bool — redstone-lamp lit state
schem.signal_strength((1, 1, 0)) # int 0..15 — wire signal level
```

`is_powered` is a single predicate that returns `True` for a lit
lamp, a powered lever or button, or any non-zero redstone signal —
the same call works whatever block is at that position.

### `simulate(sync=False)` — fast tight tick loops

The default `sync=True` keeps the existing behavior (copy the
simulator's state back into the schematic so `save()`/`render()` see
updated blocks). Passing `sync=False` skips that round-trip; query
state through the polished accessors instead. Every `simulate()` now
flushes the redpiler's pending state after `tick()` so the accessors
return current values regardless of `sync`.

### `get_block` accepts tuples

Mirrors the polished `set_block` coord shape:

```python
schem.get_block((1, 2, 3))            # in addition to (1, 2, 3)
schem.get_block_string((1, 2, 3))
```

### Custom render target

The orbit camera previously always aimed at the model's bounding-box
centroid. `render(target=(x, y, z), ...)` lets you frame a specific
corner / region of large schematics. yaw/pitch/zoom continue to
orbit around whichever target is chosen; default behaviour is
unchanged.

```python
schem.render("zoomed.png", target=(8.0, 1.0, 0.0),
             zoom=0.4, yaw=30, pitch=20)
```

## Fixed

### Mirror-symmetric torches now both update on lever toggle

A user-reported asymmetry: a wall lever feeding two redstone wires
that fan out N/S to two redstone torches mounted on solid blocks at
each end. After toggling the lever, only one torch turned off; the
other stayed lit even though both wires reached signal strength 15
symmetrically.

Tracked into MCHPRS: the redpiler's `Coalesce` pass was merging two
simulation-equivalent torch nodes (same type, same single incoming
edge from the shared lever, both removable) into one node. The
optimization is sound for the simulation graph, but each merged node
backs a *distinct world block position* whose blockstate the JIT
must sync on `flush()` — collapsing them dropped one of those
positions.

Fixed in the MCHPRS fork at commit `94f981f` (branch
`fix/torch-symmetry-non-cross-wires`): `CompileNode` now carries an
`aliased_positions` list, the `Coalesce` pass appends merged-away
positions onto the survivor, and the direct backend writes the same
block to every alias on `flush()` and `reset()`. `pos_map` resolves
each alias to the surviving NodeId so per-position lookups still
work. Per-tick simulation cost is unchanged.

Pinned in Nucleation by `tests/test_redstone_torch_symmetry.rs` —
the user's exact circuit run through the polished schematic API.

## Tests

- Python: 87 passed (TestSimulate gains 4 new cases for the cache
  contract; TestGetBlockTupleShorthand pins the tuple-input shape).
- Rust lib (sim feature): 424 passed.

---

# Nucleation v0.2.2

A correctness release for the Python simulation API plus quality-of-life
fixes across the typed circuit executor surface. CI also moves to a
tag-driven publish flow, and PyPI / npm now ship per-package READMEs.

## What's new

### Multi-step `simulate()` actually advances the wavefront

`Schematic.simulate()` previously rebuilt a fresh `MchprsWorld` on every
call. The redpiler's compile-time constant fold ran on each rebuild,
which propagated any active signals through the whole graph before any
ticks fired — `simulate(events=[lever_on], ticks=1)` followed by
`simulate(ticks=1)` would light an entire repeater chain at once instead
of advancing one delay-1 stage per call.

Fixes:

- The polished `Schematic` now caches the underlying `MchprsWorld` and
  reuses it across `simulate()` calls. Repeated `simulate(ticks=1)`
  walks the wavefront one stage per tick, as expected.
- `simulate(ticks=0, events=None)` short-circuits and returns `self`
  untouched — no more silent state mutation from a no-op call.
- New `simulate(reset=True)` and `Schematic.invalidate_simulation()`
  give you an explicit way to drop the cached world after mutating the
  schematic between runs (`set_block`, `fill`, …).
- The constant-fold leak and the cache-invalidation contract are now
  pinned by 11 dedicated tests in `TestSimulate`.

### Typed circuit executor — finishing touches

- New `TypedCircuitExecutor.sync_to_schematic()` returns a `Schematic`
  snapshot of the executor's internal world. Useful for inspecting
  block states (lit lamps, lever orientation, …) after `execute()`.
- `Value` coercion is friendlier: `as_u32` / `as_u64` now accept
  non-negative `I32` / `I64` and `Bool`, so passing `True` / `1` to a
  1-bit unsigned port "just works."
- `DefinitionRegion.with_metadata(key, value)` is now exposed as a
  fluent alias of `set_metadata` for chaining-style metadata setup.

### Per-package READMEs

PyPI no longer shows the umbrella Rust-flavoured README. The Python
package now ships `README-python.md` (pip install, Python examples,
links back to the repo); the npm package ships `README-npm.md` (npm
install, JS examples). Both are far less confusing for consumers
landing from those registries.

### CI: tag-driven publishing

The CI workflow now triggers builds on every push and PR, and only
publishes when a `v*` tag is pushed (`tags: ['v*']`). The previous
"diff Cargo.toml across HEAD^..HEAD" heuristic is gone — `check-version`
now derives the release version directly from the tag name and verifies
it matches `Cargo.toml`. Tag a release with `git tag v0.2.2 && git push
--tags` and the matrix builds + publishes everything in one run.

## Breaking

None. `simulate()` keeps its existing signature; the new `reset=` is a
keyword-only parameter that defaults to `False`.

## Tests

- Python: 108 passed (was 102; new TestSimulate cases plus the four
  pre-existing failures are now fixed).
- Rust: 258 lib tests pass.

---

# Nucleation v0.2.0

The Python API gets a structural redesign — same import name, much
nicer ergonomics, and substantially faster on the hot paths users
actually hit. JS/WASM and C/FFI gain matching upgrades. One small
behavior change is described under "Breaking" below.

## What's new

### Polished `Schematic` is a true Python subclass of the native class

`nucleation.Schematic` no longer wraps the compiled extension as a
separate Python object. It now subclasses it directly, so every native
method is available with no wrapper layer, and chains of polished calls
preserve the polished class identity:

```python
schem = (
    Schematic.new("loot_room")
        .set_block((0, 0, 0), "minecraft:chest",
                   state={"facing": "west"},
                   nbt=chest([("minecraft:diamond", 64), "minecraft:elytra"], name="Loot"))
        .set_block((0, 1, 0), "minecraft:oak_sign",
                   state={"rotation": 8},
                   nbt=sign([text("LOOT", color="gold", bold=True), "Inside"]))
)
schem.save("out.litematic")
```

### Three explicit constructors

```python
Schematic.new("name")                # blank schematic
Schematic.open("file.litematic")     # load file (format inferred)
Schematic.from_template("ab\ncd")    # ASCII template (build later via map())
Schematic("legacy.schem")            # legacy polymorphic form still works
```

### `set_block` accepts every reasonable form, in one method

The polished `set_block` lives in Rust and dispatches every shape
internally — no Python wrapper overhead.

```python
schem.set_block(0, 0, 0, "minecraft:stone")                    # 4-arg legacy
schem.set_block((0, 0, 0), "minecraft:stone")                  # tuple coords
schem.set_block((0, 0, 0), "minecraft:repeater", state={...})  # state kwargs
schem.set_block((0, 0, 0), "minecraft:chest", nbt={...})       # nbt kwargs
schem.set_block((0, 0, 0), Block("minecraft:chest", state=..., nbt=...))
```

Reusable `Block` instances cache their full payload; place the same
block at many positions without re-serializing.

### Tile-entity helpers

```python
from nucleation import chest, sign, text, Item

schem.set_block((0, 0, 0), "minecraft:chest",
                state={"facing": "west"},
                nbt=chest([
                    ("minecraft:diamond", 64),
                    "minecraft:elytra",
                    Item("minecraft:netherite_sword", components={
                        "minecraft:enchantments": {"levels": {"minecraft:sharpness": 5}},
                        "minecraft:custom_name": text("Soulrender", color="dark_purple"),
                    }),
                ], name="Loot Stash"))

schem.set_block((0, 1, 0), "minecraft:oak_sign",
                state={"rotation": 8},
                nbt=sign(["Welcome", text("LOOT", color="gold", bold=True)]))
```

### Cursor for sequential placement

```python
cursor = schem.cursor(step=(3, 0, 0))
for s in signals:
    cursor.place("minecraft:jukebox",
                 state={"has_record": True},
                 nbt={"signal": s - 1})
    cursor.place(wools[s], offset=(0, 1, 0))
    cursor.advance()
```

### Hot-loop fast path: `prepare_block` + `place`

For multi-color generative content, pre-resolve names once and place
by palette index for ~10× speedup over `set_block` calls:

```python
red  = schem.prepare_block("minecraft:red_concrete")
blue = schem.prepare_block("minecraft:blue_concrete")
place = schem.place
for x, y, z, color in voxels:
    place(x, y, z, color)
```

Available across all bindings — `prepareBlock`/`place` in JS,
`schematic_prepare_block`/`schematic_place` in FFI.

### `simulate(events=...)` collapses the redstone round-trip

The 5-step `create_simulation_world` → `on_use_block` → `tick` →
`sync_to_schematic` → `get_schematic` flow becomes one call:

```python
from nucleation import UseBlock

schem.simulate(ticks=4, events=[UseBlock((0, 1, 3))])
```

The legacy `create_simulation_world()` API still works for multi-stage
flows.

### One-shot simulate convenience in FFI

`schematic_simulate_use_block(handle, ticks, events_xyz, n_events)`
runs the same end-to-end flow from C without managing a separate
world handle.

### Format inference for `save()` / `export_mesh()`

```python
schem.save("out.litematic")   # litematic
schem.save("out.schem")       # sponge schematic
schem.save("out.mcstructure") # bedrock mcstructure
schem.save("/tmp/x", format="schem")  # explicit override

schem.export_mesh("out.glb")  # or .nucm
```

### Render with kwargs

```python
schem.with_pack(pack)
schem.render("out.png", width=3840, height=2160, yaw=45, pitch=45, zoom=0.7)
# or:
schem.render("out.png", config=RenderConfig(...))
```

## Performance

Real workloads (release builds, measured against the pre-session 0.1.x
baseline on the same hardware):

### Python — 200,337-voxel Mandelbulb (`bench_python.py`)

| Operation | Before | After | Speedup |
|---|---:|---:|---:|
| `set_block` loop, 16 colors | 5.0 M/s | 4.8 M/s | within ~3% |
| `set_blocks` batch | 14.9 M/s | 32 M/s | **2.1×** |
| `prepare_block + place` (new) | n/a | 9 M/s | **1.8× over old set_block** |
| `fill_cuboid` | 82 M/s | 1,100 M/s | **13×** |
| `flip_x` (5k chests) | n/a | 80 M/s | huge improvement on transforms |
| `rotate_y 90°` (5k chests) | n/a | 89 M/s |  |
| Schematic clone (10k chests) | n/a | 240 M/s |  |

### Python — 1M chests with NBT (`set_blocks` parse-once batch)

| Stage | Throughput | 1B chests ETA |
|---|---:|---:|
| Pre-session per-call SNBT loop | 28 K/s | ~10 hours |
| Final, after structural changes | **20 M/s** | **~50 seconds** |

The chest-batch case improved ~700× thanks to:
- Parse-once batch path (one SNBT parse, applied to N positions)
- `BlockEntityStore` (palette of `Arc<BlockEntity>` templates +
  position index, with `insert_template` fast path that shares one
  Arc across all positions)
- `Arc<NbtMap>` so cloning a `BlockEntity` is a refcount bump (~16 ns)
  instead of a deep tree walk (~250 ns)
- FxHashMap on `palette_index` and `block_state_cache`

### JS / WASM — per-block placement

WASM has dramatically lower per-call overhead than PyO3:

| API | Throughput |
|-----|---:|
| `s.raw.set_block` | 10.8 M/s |
| `s.setBlock([x,y,z], "id")` | 7.7 M/s |
| `prepareBlock` + `place` (single id) | 60.6 M/s |
| `prepareBlock` + `place` (16 colors) | **85.8 M/s** |

JS users hitting the prepare+place pattern get near-native-Rust speed.

## Errors

Every binding gained tighter, tested error messages. Selected
improvements:

- Python: `set_block(1, 2)` now says `first arg must be a 3-tuple of
  ints` instead of leaking `'int' object cannot be converted to
  'PyTuple'`.
- Python: `nbt="raw_string"` now raises `TypeError` (was silently
  accepted, hiding typo bugs). Use `nbt={"__snbt__": "..."}` for raw
  SNBT.
- Python: `Schematic.new(123)` says `name must be a string`.
- Python: `Block.parse(123)` says `expects a string`.
- Python: `export_mesh("x.bad")` reports the bad extension before
  complaining about a missing pack.
- JS: matching validation across the board, including bare-string nbt
  rejection.
- FFI: `schematic_prepare_block` / `schematic_place` set thread-local
  `last_error` with actionable messages on null handles, negative
  indices, or out-of-range palette indices.

35 new error-handling tests across the bindings (18 pytest, 12 node, 5 ffi).

## Breaking

Only one — and it caught a class of silent bugs:

```python
# Used to silently slip through into the SNBT parser:
schem.set_block((0, 0, 0), "minecraft:chest", nbt="raw_string")
# Now raises TypeError. Either pass a real dict:
schem.set_block((0, 0, 0), "minecraft:chest", nbt={"Items": [...]})
# ...or use the explicit __snbt__ escape hatch:
schem.set_block((0, 0, 0), "minecraft:chest", nbt={"__snbt__": "{...}"})
```

All other previously-working call patterns continue to work unchanged
(legacy 4-arg `set_block`, `Schematic("name.schem")` polymorphic form,
the deprecated `SchematicBuilder` shim with `DeprecationWarning`,
every native method like `set_block_with_properties`,
`set_block_from_string`, `fill_cuboid`, format I/O, etc.).

## Tests

- cargo test (default): 401 passing
- cargo test --features simulation: 596 passing
- pytest: 71 passing (was 53)
- node: 45 passing (was 30)
- FFI helpers: 17 passing (was 12)
- FFI simulate: 3 passing
- API parity check: 0 missing across all bindings

Pre-push: 15/15 functional checks passing.

# Nucleation v0.1.183

Ships prebuilt Python wheels for every major platform, not just Linux
x86_64. Windows + macOS users can now `pip install nucleation` without a
local Rust toolchain.

Wheels are built per-platform in CI via `PyO3/maturin-action` and then
uploaded together by the publish job. The matrix:

| Platform | Target |
|---|---|
| Linux x86_64  | `x86_64-unknown-linux-gnu` (manylinux) |
| Linux aarch64 | `aarch64-unknown-linux-gnu` (manylinux, cross) |
| macOS arm64   | `aarch64-apple-darwin` |
| macOS x86_64  | `x86_64-apple-darwin` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

The source distribution (sdist) is still uploaded so anything outside
that matrix (e.g. Linux armv7, exotic Windows targets, experimental
Python builds) can still fall back to `pip install --no-binary` source
builds.

# Nucleation v0.1.182

Bumps `schematic-mesher` to pick up the GLB greedy-meshing UV fix.
Production GLBs were rendering merged faces with the entire texture
atlas smeared across them; the mesher now preserves per-greedy-material
textures end-to-end so each merged quad samples its own tile with REPEAT
wrapping. Use `MeshConfig::default().with_greedy_meshing(true)` and
re-export to confirm.

# Nucleation v0.1.181

Adds a loader for the legacy MCEdit / Classic `.schematic` format, bumps
`schematic-mesher` to pick up 1.21.5 entity models, and re-ships everything
that didn't make it to PyPI in 0.1.180.

## Classic `.schematic` format — import only

Old community builds shipped as pre-1.13 `.schematic` files (numeric block
IDs + 4-bit metadata, `Materials: "Alpha"`) now load through the normal
FormatManager path:

```rust
let bytes = std::fs::read("old_build.schematic")?;
let schem = nucleation::formats::manager::get_manager()
    .lock().unwrap()
    .read(&bytes)?;                  // auto-detected
```

The importer handles:
- `Blocks` + `Data` + optional `AddBlocks` (nibble-packed upper 4 bits of
  the block ID for values > 255)
- TileEntities — rewrites the legacy `id`+`x`+`y`+`z` schema to the
  modern `Id`+`Pos` one before delegating to `BlockEntity::from_nbt`
- Entities via `Entity::from_nbt`
- A ~150-entry 1.12 → modern block-state mapping covering stone variants,
  wood, wool, slabs, stairs, logs, leaves, glass, stained glass, sandstone,
  quartz, fences/gates, ores, etc. Unknown ids fall through to
  `minecraft:stone` so geometry is preserved.

No exporter — the format is deprecated; round-trip out to Sponge v3,
Litematica, or mcstructure instead.

## Mesher bump — 1.21.5 entity models

`schematic-mesher` updated to pull in the MC 1.21.5 entity model port
(boats, mounts, equipment, skin resolver, villager layering, sign text
rendering, decorated pots), plus an atlas fallback tile that makes
missing-texture cases visible (magenta/black checkerboard + stderr
warning) and a regression test for the greedy-tile UV leak seen in
production GLBs.

## Republish of 0.1.180 to PyPI

v0.1.180 went to crates.io and npm successfully but the PyPI step
failed because `pyproject.toml` was left at 0.1.179, so maturin built a
0.1.179 wheel that PyPI rejected as a duplicate. This release catches
PyPI up.

> **Note:** v0.1.173–v0.1.175 were partially published (crates.io only)
> while the CI publish pipeline was being migrated to npm Trusted Publishing.
> v0.1.180 landed on crates.io and npm but skipped PyPI due to the version
> mismatch above. 0.1.181 is fully published everywhere.

## Minecraft Item Model Export

Convert any schematic into a Minecraft resource pack that renders as a custom 3D item model in-game.

### Overview

Nucleation can now generate Minecraft-compatible item models from schematics using a hybrid rendering approach. Full-cube blocks are composited into efficient plane-based slices (max 288 elements for a 48x48x48 build), while non-full-cube blocks (levers, torches, redstone, stairs, etc.) are rendered as individual model elements with their actual 3D geometry. The result is a ready-to-use resource pack that can be loaded directly into Minecraft 1.21.4+.

### How it works

1. The schematic is sliced into 2D planes for each of the 6 directions (north, south, east, west, up, down)
2. For each plane, visible block face textures are resolved through the full blockstate -> model -> texture chain using an existing resource pack
3. Face textures are composited into a single PNG per plane, with correct UV rotation for rotated blocks (levers, torches, etc.)
4. Non-full-cube blocks are extracted as individual model elements preserving their original shape and rotation
5. A Minecraft item model JSON is generated referencing all plane and element textures
6. Everything is packaged into a resource pack ZIP with proper `pack.mcmeta`, item definitions, model JSON, and texture PNGs

### Usage

### Python

```python
import nucleation

schematic = nucleation.Schematic.from_file("my_build.schem")
pack = nucleation.ResourcePack.from_file("1.21.4.zip")

config = nucleation.ItemModelConfig("my_build")
config.item = "paper"                # Minecraft item to bind to
config.custom_model_data = "1"       # Selector value

result = schematic.to_item_model(pack, config)
result.save_resource_pack("my_pack.zip")
```

To merge multiple schematics into a single resource pack:

```python
configs = [
    ("build_a", "1"),
    ("build_b", "2"),
    ("build_c", "3"),
]

results = []
for name, cmd in configs:
    config = nucleation.ItemModelConfig(name)
    config.custom_model_data = cmd
    results.append(schematic.to_item_model(pack, config))

nucleation.build_resource_pack(results, "merged_pack.zip")
```

Then in Minecraft:
```
/give @s minecraft:paper[custom_model_data={strings:["1"]}]
```

### JavaScript (WASM)

```js
import { Schematic, ResourcePack, ItemModelConfig, buildResourcePack } from "nucleation";

const schematic = Schematic.fromBytes(data, "schem");
const pack = new ResourcePack(packBytes);

const config = new ItemModelConfig("my_build");
config.item = "paper";
config.customModelData = "1";

const result = schematic.toItemModel(pack, config);
const zip = result.toResourcePackZip();

// Or merge multiple results
const mergedZip = buildResourcePack([result1, result2, result3]);
```

### C/C++ (FFI)

```c
NucItemModelConfig* cfg = itemmodel_config_new("my_build");
itemmodel_config_set_item(cfg, "paper");
itemmodel_config_set_custom_model_data(cfg, "1");

NucItemModelResult* result = schematic_to_item_model(schematic, pack, cfg);

// Single result -> ZIP
uint32_t zip_len;
uint8_t* zip = itemmodel_result_to_resource_pack_zip(result, &zip_len);

// Or merge multiple results
NucItemModelResult* results[] = {r1, r2, r3};
uint8_t* merged = itemmodel_build_resource_pack(results, 3, &zip_len);
```

### Configuration

| Field | Default | Description |
|-------|---------|-------------|
| `model_name` | `"schematic"` | Name used in file paths and texture references |
| `namespace` | `"nucleation"` | Resource pack namespace |
| `center` | `true` | Center the schematic within model coordinate bounds |
| `texture_resolution` | `16` | Pixels per block face |
| `item` | `"paper"` | Minecraft item to bind the model to |
| `custom_model_data` | `"1"` | Value for selecting the model via `custom_model_data` component |

### Result stats

`ItemModelResult` includes statistics about the generated model:

- `plane_count` - number of plane elements (efficient composited slices)
- `element_count` - total elements including individual non-full-cube blocks
- `texture_count` - number of generated texture PNGs
- `dimensions` - schematic dimensions as `[x, y, z]`

### Constraints

- Maximum schematic size: 48x48x48 blocks (Minecraft model coordinate range: -16 to 32)
- Requires a Minecraft resource pack (1.21.4+) for texture resolution
- Generated resource packs target Minecraft 1.21.4+ item model format (`pack_format: 46`)

## Entity Fixes

Two bugs that caused entities to be silently dropped at export time have been
fixed.

### Schematic bounding box now tracks entity positions

Previously, `add_entity` / `add_block_entity` updated only the entity list and
not the schematic's tight bounds. `to_compact()` (used by the `.litematic` and
`.schem` exporters) then filtered out any entity whose position fell outside
that bbox — so entities placed outside the block envelope were quietly lost in
the exported file, and Litematica would also filter them on load.

The new `Region::get_content_bounds()` (plus matching
`UniversalSchematic::get_content_bounds()`) returns the union of non-air
blocks, entity positions, and block-entity positions. `to_compact()` now sizes
the compact region from content bounds and keeps every entity that was added.
`litematic::to_litematic` uses content bounds for `EnclosingSize` so Litematica
loads all entities correctly.

### `.schem` entity NBT now matches the Sponge v3 spec

`convert_entities` previously wrote the vanilla Minecraft chunk-format entity
NBT directly (lowercase top-level `id`) and an internal validator silently
dropped any entity whose compound lacked a capital `Id`, which meant every
entity was discarded at export time. The reader had a matching bug.

Export now emits v3-compliant wrappers — `{ Id, Pos, Data: { vanilla MC entity
NBT } }` — with `Motion`/`Rotation` defaults populated in `Data` so
spec-compliant loaders (WorldEdit, FastAsyncWorldEdit, etc.) accept the file.
v2 export emits the flat layout defined by the v2 spec. `parse_entities`
transparently handles both shapes, so existing `.schem` files produced by
older Nucleation versions still load.

## Multi-pack Resource Pack Loading

`ResourcePackSource` now accepts one *or* multiple resource packs, matching
Minecraft's own pack-priority model (lowest priority first, each subsequent
pack overlays the previous on per-key collision). This is additive — the
existing `from_file` / `from_bytes` entry points are unchanged.

### Rust

```rust
use nucleation::meshing::ResourcePackSource;

// Single pack (existing API — unchanged)
let pack = ResourcePackSource::from_file("vanilla.zip")?;

// Multiple packs — priority order, lowest first
let pack = ResourcePackSource::from_files([
    "vanilla.zip",
    "base_mod.zip",
    "texture_pack_override.zip",
])?;

// Or from byte buffers (WASM-friendly)
let pack = ResourcePackSource::from_bytes_list(vec![vanilla_bytes, override_bytes])?;
```

### Python

```python
pack = nucleation.ResourcePack.from_files(["vanilla.zip", "override.zip"])
# or
pack = nucleation.ResourcePack.from_bytes_list([vanilla_bytes, override_bytes])
```

### JavaScript (WASM)

```js
const pack = ResourcePackWrapper.fromBytesList([vanillaBytes, overrideBytes]);
```

### C/C++ (FFI)

```c
const uint8_t* ptrs[] = { vanilla_bytes, override_bytes };
size_t        lens[] = { vanilla_len,   override_len };
NucResourcePack* pack = resourcepack_from_bytes_list(ptrs, lens, 2);
```

The merged pack is a drop-in for every mesher entry point — `to_mesh`,
`to_usdz`, `to_item_model`, `build_global_atlas`, etc. — so you can load once
and reuse across outputs.

Built on top of the corresponding upstream addition in `schematic-mesher`
(`ResourcePack::overlay`, `load_resource_packs`, `load_resource_packs_from_bytes`),
available on the `feature/mesh-output-api` branch.
