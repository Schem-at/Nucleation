# Nucleation for Python

[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)

Python bindings for **Nucleation**, a high-performance Minecraft schematic
engine. Read, edit, build, simulate, and mesh `.schematic`, `.litematic`,
`.nbt`, and `.mcstructure` files — at native speed.

The core is written in Rust and compiled to a CPython extension via PyO3
(`abi3-py38`), so a single wheel works on every CPython 3.8+ on your
platform — no Rust toolchain required to install.

---

## Install

```bash
pip install nucleation
```

Pre-built wheels are published for:

- Linux x86_64 and aarch64
- macOS x86_64 (Intel) and arm64 (Apple Silicon)
- Windows x86_64

If your platform isn't covered, `pip` will fall back to the source
distribution and build locally (requires a Rust toolchain).

---

## Quick start

```python
from nucleation import Schematic, sign, text

# Three explicit constructors (legacy `Schematic("file.schem")` still works):
schem = Schematic.open("example.litematic")        # load from disk
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

### Hot loops

For placing many blocks, prefer the batch and fast-path APIs:

```python
# 30+ M placements/sec — one native call.
schem.set_blocks([(x, 0, 0) for x in range(1_000_000)], "minecraft:stone")

# 10+ M placements/sec — pre-resolve once, place by index.
stone = schem.prepare_block("minecraft:stone")
place = schem.place
for x, y, z in positions:
    place(x, y, z, stone)
```

### Tile-entity helpers

```python
from nucleation import Schematic, chest, sign, text, Item

schem = Schematic.new("loot_room")

schem.set_block((0, 0, 0), "minecraft:chest",
                state={"facing": "north"},
                nbt=chest([
                    Item("minecraft:diamond", count=64),
                    Item("minecraft:elytra"),
                ]))

schem.set_block((0, 1, 0), "minecraft:oak_sign",
                nbt=sign([text("Loot", color="gold"), "this way →"]))
```

For lossless, fully-typed reads/writes (64-bit NBT, item components), use the
SNBT accessors: `get_block_entity_snbt(x, y, z)` / `set_block_entity(x, y, z, id, snbt)`
and `get_entities_snbt()` / `add_entity_from_snbt(snbt)`.

### Version conversion (datafixers)

Convert block / block-entity / item / entity data **between Minecraft data
versions** — a Rust port of PaperMC's DataConverter. Forward conversion is
lossless; down-converting to an older version returns a JSON **loss report** so
you can warn before saving. Importers capture the source data version (set it
manually for versionless formats; classic `.schematic` ≈ `1343`).

```python
import json

schem = Schematic.open("modern.litematic")
print(schem.get_source_data_version())              # e.g. 3953 (1.21)
print(Schematic.canonical_data_version())           # 4790 (in-memory target)

# Save a copy for 1.16.5 (data version 2586); schem itself is untouched.
data, loss = schem.to_litematic_for_version(2586)
report = json.loads(loss)                            # [] when lossless
for e in report:                                    # {version, kind, severity, path, detail}
    print(f"[{e['severity']}] {e['path']}: {e['detail']}")

# Or convert in place using the captured source version:
loss = schem.convert_to_version(Schematic.canonical_data_version())
```

### Remote storage

`Schematic.open` and `schem.save` work transparently against remote backends —
no extra object, no manual byte-shuffling. The same calls you use for local
files take a scheme'd URI or an explicit `Store`:

```python
from nucleation import Schematic, Store

# local file (unchanged)
schem = Schematic.open("build.schem")
schem.save("build.litematic")

# transparent remote: a scheme'd URI, no Store object needed
schem = Schematic.open("s3://my-bucket/builds/adder.schem")
schem.save("s3://my-bucket/out/adder.schem")
schem.save("file:///tmp/adder.litematic")   # file:// = local path

# explicit Store + key: works for ANY backend (incl. redis/postgres)
# and reuses a single connection
store = Store.open("redis://localhost:6379")
schem = Schematic.open(store, "builds/adder.schem")
schem.save(store, "out/adder.schem")
```

The format is inferred from the key/path extension (`.schem`, `.litematic`,
`.mcstructure`); pass `format="litematic"` etc. to `save` to override it.

The single-string form works for path-like backends — `s3://bucket/key` (and
`file://` → local). `redis://` and `postgres://` single-strings raise
`ValueError`, because their URL path is the database, leaving no slot for the
key; open those with an explicit store instead:
`Schematic.open(Store.open(url), key)`.

Which backends are reachable depends on the `store-*` features the extension was
built with. The default wheel ships `mem://` + `file://`; for S3/Redis/Postgres
rebuild it:

```bash
maturin develop --features python,store-s3,store-redis,store-pg
```

**Raw byte store.** When you want raw bytes rather than schematics — PNG renders,
arbitrary artifacts — use the [`Store`](docs/api-reference-python.md#store) class
directly (`get`/`put`/`exists`/`delete`/`list`/`health`).

---

## Diff & Fingerprint

Canonically fingerprint a build, dedup near-duplicates, and compute a structural
diff between two builds — all under a configurable equivalence ruleset chosen by
**preset name**:

- `exact` — material- and orientation-sensitive (identical blockstates only).
- `shape` — occupancy only; palette and orientation ignored.
- `structural` — functional shape, rotation- and material-agnostic.
- `redstone_computational` (alias `redstone`) — redstone-logic equivalence,
  rotation-agnostic, cosmetic-material-agnostic.
- `redstone_survival` — like `redstone`, but keeps survival material constraints.

`diff` additionally accepts opt-in overrides: per-edit cost weights
(`cost_add` / `cost_delete` / `cost_change` / `cost_swap`) and a `symmetry` group.

```python
from nucleation import Schematic

a = Schematic.open("a.litematic")
b = Schematic.open("b.litematic")

# 32-hex canonical hash (rotation/translation/palette-agnostic per preset).
print(a.fingerprint("structural"))

# Cheap exact-equivalence dedup, and fuzzy FFT shape distance (0.0 == same shape).
if a.is_duplicate_of(b, "structural"):
    print("duplicate build")
print("footprint distance:", a.footprint_distance(b, "structural"))

# Dims + token histogram as JSON.
print(a.signature("structural"))

# Structural diff (optionally pass cost_*/symmetry overrides).
d = a.diff(b, "redstone")
print("distance:", d.distance())
# support = fraction of the larger build's cells that aligned (confidence,
# NOT a similarity %).
print("support:", d.support())

# Each delta as its own Schematic.
d.added(); d.removed(); d.changed(); d.swapped(); d.markers()

# Lossless JSON round-trips via Diff.from_json; summary_json() is compact.
from nucleation import Diff
full = d.to_json()
restored = Diff.from_json(full)
print(d.summary_json())
```

The glowing-overlay GLB requires the `meshing` feature in your wheel:

```python
# `after_glb` is the meshed "after" build (bytes); returns a new GLB.
glb = d.to_overlay_glb(after_glb)
with open("diff_overlay.glb", "wb") as f:
    f.write(glb)
```

---

## Documentation

Full reference, including simulation, meshing, and resource-pack rendering,
lives in the main repository:

- [Nucleation on GitHub](https://github.com/Schem-at/Nucleation)
- [Python API reference](https://github.com/Schem-at/Nucleation/blob/master/docs/api-reference-python.md)
- [Schematic Builder guide](https://github.com/Schem-at/Nucleation/blob/master/docs/guide/schematic-builder.md)

---

## Why Rust?

Nucleation's core is shared across Rust, WebAssembly/JS, Python, and C/PHP —
one engine, one set of behaviour, one set of tests. The Python package you
install is the same engine that powers the JS and Rust libraries; you get
native throughput for free.

## License

MIT. See the [main repository](https://github.com/Schem-at/Nucleation)
for details.
