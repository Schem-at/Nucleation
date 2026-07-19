# Nucleation documentation

Start with the [project README](../README.md) — installation, the basics, and
an illustrated tour. Everything here goes deeper.

Since v0.3.0 every binding is generated from one source of truth
(`src/bridge/`), so **the API is the same everywhere**: same types, same
methods, per-language casing (`set_block` in Rust/Python, `setBlock` in
JS/Kotlin/PHP), unified `NucleationError` errors.

## Feature guides

- [Shapes, brushes, and masked fills](guides/shapes-and-brushes.md)
- [Palettes: turning colors into blocks](guides/palettes.md)
- [SDF shapes and terrain](guides/sdf-terrain.md)
- [Embedded scripting (Lua / JS)](guides/scripting.md)
- [Auto-stack: detect and resize repetition](autostack.md)
  ([design notes](autostack-design.pdf))
- [The Minecraft block database](guides/minecraft-block-data.md) — data
  provenance, the 26.2 refresh pipeline, Bedrock mappings
- [Insign IO integration](insign-io-integration.md) — executors from sign
  annotations
- [Meshing, .nucm, and rendering](meshing-nucm-rendering.md)

## More capabilities

Covered in the README's illustrated tour, each with a verified Python snippet:

- [Voxelize 3D models](../README.md#voxelize-3d-models) — GLB/OBJ → building
  `Shape`s, texture projection, and surface-only voxelization (negative
  `shell`) for open ribbons that dip or self-overlap
- [Read, iterate, and stream](../README.md#read-iterate-and-stream) — chunk
  iteration strategies + the `WorldStream`/`WorldSink` constant-memory world
  pipeline ([snippet](readme-snippets/12-chunk-iteration-python.md)), and the
  `to_schematic` ↔ `from_schematic` bridge that makes any fill (SDF, OSM,
  heightmap, noise) a custom world **generator** or **filter**
  ([snippet](readme-snippets/17-world-generator-python.md))
- [Regions, transforms & stamping](../README.md#regions-transforms-and-stamping)
  — multi-region schematics, rotate/flip, `copy_region`
  ([snippet](readme-snippets/13-regions-transforms-python.md))
- [Block entities, entities & NBT](../README.md#block-entities-entities-and-nbt)
  ([snippet](readme-snippets/14-block-entities-nbt-python.md))
- [Geodata](../README.md#the-real-world-in-blocks) — `Shape.polygon_prism`,
  `Geo.extrude_footprints`, `Geo.heightmap_terrain`, then out to a playable
  world ([snippet](readme-snippets/16-geo-osm-python.md))
- [Pluggable storage](../README.md#pluggable-storage) — `StoreIo` / `Store`
  over one URI (memory / filesystem / S3 / Redis / Postgres)
  ([snippet](readme-snippets/15-storage-python.md))

## Per-language references

- [Rust](rust/) · [JavaScript](javascript/) · [Python](python/) ·
  [Kotlin](kotlin/) · [PHP](php/) · [C](c/) · [C++](cpp/)

## Verified examples

Every snippet in the README ran for real with captured output:
[`docs/readme-snippets/`](readme-snippets/). The README's images regenerate
from [`tools/readme-media/generate.py`](../tools/readme-media/generate.py).

## Formats

`.litematic` · Sponge `.schem` · WorldEdit `.schematic` · Bedrock
`.mcstructure` · structure `.nbt` · `.nusn` (fast binary snapshot) — with
auto-detection — plus world folders (Anvil region files) in both directions.

## License

MIT — see [LICENSE](../LICENSE).
