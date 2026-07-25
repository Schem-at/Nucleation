# Nucleation documentation

Start with the [project README](../README.md) — installation, the feature index,
and the gallery. Everything here goes deeper.

Since v0.3.0 every binding is generated from one source of truth
(`src/bridge/`), so **the API is the same everywhere**: same types, same
methods, per-language casing (`set_block` in Rust/Python, `setBlock` in
JS/Kotlin/PHP), unified `NucleationError` errors.

## Feature guides

- [Basics](features/basics.md) — create a schematic, place and inspect blocks,
  coordinates, automatic growth, and block-state strings
- [Formats and I/O](features/formats-and-io.md) — format detection, conversion,
  round-trip fidelity, and filesystem/byte APIs
- [Regions, transforms & stamping](features/regions-and-transforms.md)
  — region lifecycle, scoped rigid transforms, `stamp_box` / `stamp_region`
  ([snippet](readme-snippets/13-regions-transforms-python.md))
- [Shapes, brushes, and masked fills](features/shapes-and-brushes.md)
- [Palettes: turning colors into blocks](features/palettes-and-color.md)
- [SDF shapes and terrain](features/sdf-and-fields.md)
- [Embedded scripting (Lua / JS)](features/scripting.md)
- [Auto-stack: detect and resize repetition](features/analysis.md)
  ([design notes](autostack-design.pdf))
- [The Minecraft block database](features/block-database.md) — data
  provenance, the 26.2 refresh pipeline, Bedrock mappings
- [Insign IO integration](features/redstone-simulation.md) — executors from sign
  annotations
- [Meshing, .nucm, and rendering](features/meshing-and-rendering.md)
- [Animating a build](features/animation.md) — assembly, layer printing,
  reveals along a shape's own curve; deterministic frame sampling

## More capabilities

Each with a verified Python snippet:

- [Voxelize 3D models](features/voxelize.md) — GLB/OBJ → building
  `Shape`s, texture projection, and surface-only voxelization (negative
  `shell`) for open ribbons that dip or self-overlap
- [Read, iterate, and stream](features/streaming-and-worlds.md) — chunk
  iteration strategies + the `WorldStream`/`WorldSink` constant-memory world
  pipeline ([snippet](readme-snippets/12-chunk-iteration-python.md)), and the
  `to_schematic` ↔ `from_schematic` bridge that makes any fill (SDF, OSM,
  heightmap, noise) a custom world **generator** or **filter**
- [World segmentation](features/world-segmentation.md) — turn a whole world
  save into individual builds: learned-substrate subtraction, morphological
  clustering, cross-tile stitching, tiers, partition hints, snapshot identity,
  and a provenance envelope per build — byte-identical on every re-run
  ([snippet](readme-snippets/17-world-generator-python.md))
- [Block entities, entities & NBT](features/block-entities-nbt.md)
  ([snippet](readme-snippets/14-block-entities-nbt-python.md))
- [Geodata](features/geo.md) — `Shape.polygon_prism`,
  `Geo.extrude_footprints`, `Geo.heightmap_terrain`, then out to a playable
  world ([snippet](readme-snippets/16-geo-osm-python.md))
- [Pluggable storage](features/storage.md) — `StoreIo` / `Store`
  over one URI (memory / filesystem / S3 / Redis / Postgres)
  ([snippet](readme-snippets/15-storage-python.md))

## Per-language references

- [Rust](rust/) · [JavaScript](javascript/) · [Python](python/) ·
  [Kotlin](../bindings/kotlin/) · [PHP](../bindings/php/) ·
  [C](../bindings/c/) · [C++](../bindings/cpp/)

## Verified examples

Every snippet in the README ran for real with captured output:
[`docs/readme-snippets/`](readme-snippets/). The README's images regenerate
from [`tools/readme-media/generate.py`](../tools/readme-media/generate.py).

## Formats

`.litematic` · Sponge `.schem` · legacy MCEdit `.schematic` (import only) ·
Bedrock `.mcstructure` · `.nusn` (fast binary snapshot) — with content-based
auto-detection — plus Anvil region files, zipped worlds, and world directories.

## License

MIT — see [LICENSE](../LICENSE).
