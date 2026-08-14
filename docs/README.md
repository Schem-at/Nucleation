# Nucleation documentation

The [documentation home](index.md) contains installation commands, a measured
specimen, and the shortest route into each subsystem.

Rust owns the implementation. Since v0.3.0, `src/bridge/` has defined the API
for Python, JavaScript, Kotlin, PHP, C, and C++. Generated bindings keep the same
types and methods while following local naming conventions. Rust and Python use
`set_block`; JavaScript and Kotlin use `setBlock`. Binding errors share the
`NucleationError` model where the target language permits it.

## Start here

| Path | Subject |
|---|---|
| [Basics](features/basics.md) | Create a schematic, place blocks, inspect state, and save the result. |
| [Formats and I/O](features/formats-and-io.md) | Detect input by content, choose an exporter, and state the losses before conversion. |
| [Bindings](features/bindings-and-languages.md) | Package names, error forms, naming, and target-specific limits. |
| [Gallery](gallery.md) | Rendered output with source and downloadable schematics. |

## Construction

| Path | Subject |
|---|---|
| [Regions and transforms](features/regions-and-transforms.md) | Region lifecycle, rigid transforms, `stamp_box`, and `stamp_region`. |
| [Shapes and brushes](features/shapes-and-brushes.md) | Primitive geometry, boolean composition, masked fills, and material choice. |
| [Palettes and colour](features/palettes-and-color.md) | Measured block colours, ramps, matching, and dithering. |
| [SDFs and fields](features/sdf-and-fields.md) | Typed scalar fields, bounded sampling, terrain, Voronoi, and noise. |
| [Voxelization](features/voxelize.md) | GLB and OBJ input, texture projection, and surface-only conversion. |
| [Geodata](features/geo.md) | Elevation grids and OpenStreetMap footprints. |

## Worlds and stored data

| Path | Subject |
|---|---|
| [Streaming and worlds](features/streaming-and-worlds.md) | Chunk iteration and constant-memory world input and output. |
| [World segmentation](features/world-segmentation.md) | Substrate subtraction, clustering, stitching, tiers, and provenance. |
| [Schematic provenance](features/schematic-provenance.md) | Source identity, map coordinates, dimensions, and world metadata. |
| [Transformation policies](features/transformation-policies.md) | Bounded decoding, normalization, content rules, audit history, and routing. |
| [Block entities and NBT](features/block-entities-nbt.md) | Typed SNBT round trips for blocks and entities. |
| [Storage](features/storage.md) | Memory, filesystem, SSH, S3, Redis, and Postgres backends under one interface. |

## Execution and output

| Path | Subject |
|---|---|
| [Redstone simulation](features/redstone-simulation.md) | Compiled circuit execution and typed Insign I/O. |
| [Tick simulation](features/tick-simulation.md) | Game-order ticks, updates, fluids, pistons, entities, and snapshots. |
| [Meshing and rendering](features/meshing-and-rendering.md) | NUCM, GLB, glTF, USDZ, and the headless renderer. |
| [Animation](features/animation.md) | Groups, tracks, assembly order, layer printing, and deterministic sampling. |
| [Analysis](features/analysis.md) | Diff, fingerprint, repetition detection, and auto-stack. |
| [Scripting](features/scripting.md) | Lua and JavaScript execution against the schematic API. |

## Language references

[Rust](rust/) · [JavaScript](javascript/) · [Python](python/) ·
[Kotlin](../bindings/kotlin/) · [PHP](../bindings/php/) ·
[C](../bindings/c/) · [C++](../bindings/cpp/)

## Verified material

The snippets under [`docs/readme-snippets/`](readme-snippets/) include captured
output. Images used by the overview regenerate from
[`tools/readme-media/generate.py`](../tools/readme-media/generate.py).

Supported containers include `.litematic`, Sponge `.schem`, Bedrock
`.mcstructure`, Java structure `.snbt`, and `.nusn`. Legacy MCEdit `.schematic`
is import-only. World I/O also accepts Anvil regions, zipped worlds, and world
directories.

## Licence

MIT. See [LICENSE](../LICENSE).
