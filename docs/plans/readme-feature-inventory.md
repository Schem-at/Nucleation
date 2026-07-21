# Nucleation feature inventory

Derived from `Cargo.toml` features, `src/` modules, and `src/bridge/` (the
generated public API surface) — not from the README, so gaps are visible.

Legend: **★** = currently showcased in README · **○** = in code, absent/buried in README

## 1. Core model & I/O
- ★ `UniversalSchematic`: multi-region, block entities, entities, metadata
- ★ Formats: litematic, `.schem` (Sponge), `.schematic` (classic/MCEdit), `.mcstructure` (Bedrock), Anvil worlds, snapshot
- ★ Block-state strings with properties, full round-trip
- ★ Named regions with own palette/bounds; `rotate_region_y`, per-region addressing
- ★ Transforms: `rotate_x/y/z`, `flip_x/y/z`, `copy_region` stamping
- ★ Chunk iteration with traversal strategies (bottom_up, center_outward, distance_to_camera, random)
- ★ `WorldStream`/`WorldSink`: constant-memory world streaming; two-way chunk↔schematic bridge
- ★ Worlds: `save_world`, `from_world_directory_bounded`
- ○ `selection`: connected-component flood selection over block coords
- ○ `DefinitionRegion`: logical region from multiple disjoint bounding boxes (circuit inputs/outputs)
- ○ `item` module

## 2. Building & generation
- ★ Shapes: sphere, cuboid, ellipsoid, cylinder, cone, torus, pyramid, disk, plane, triangle, line, bezier, hollow, composite, polygon_prism
- ★ Brushes: solid, linear/curve gradient, shaded (normal-lit), spotlight, field
- ★ Masked fills: `fill_only_air`, `fill_replacing`
- ★ SDF: JSON-described fields; any SDF tree is a `Shape`; smooth booleans, displace, warp, repeat
- ★ SDF terrain: declarative material rules (surface shells, depth bands, gradients, scatter)
- ★ `cells` Worley/Voronoi field (`f1`, `f2`, `f2MinusF1`)
- ★ `DistanceField`: depth + surface normal over *any* build
- ★ Voxelize: GLB/OBJ → schematic, texture projection, scanline voxelizer
- ★ Geo: `heightmap_terrain` (elevation grids), `extrude_footprints` (OSM)
- ○ `SchematicBuilder`: ASCII/layer art → schematic (mentioned in one line)
- ○ CLI binary: `schematic-builder`

## 3. Color & palettes
- ★ Blockpedia: 1,196 MC 26.2 blocks — kinds, variant families, tags, geometry, measured colors; self-updating
- ★ 7 palette presets; `gradient_ids_json`, `ramp_ids_json`
- ★ Oklab interpolation; ordered Bayer dithering
- ★ `PaletteBuilder` color-logic filters (chroma, lightness, full-blocks-only)
- ★ Color matching: `by_color_json`, `closest_block_dithered`, chroma boost
- ○ k-means / median color extraction methods

## 4. Analysis
- ★ Diff: added/removed/changed/swapped, region grouping, palette-swap collapsing, overlay GLB
- ★ Fingerprint: `exact`/`shape`/`structural`/`redstone*` presets; translation- and rotation-invariant dedup
- ★ Autostack: detect repeating lattice, resize
- ○ Signature (cheap invariant prefilter) and FFT `Footprint` distance
- ○ Block-entity NBT sensitivity (new, `FingerprintSpec::block_entities`)

## 5. Redstone simulation
- ★ MCHPRS redpiler, headless; runs in WASM
- ★ `TypedCircuitExecutor`: named typed I/O (bool/int/float/ASCII) + bus layout builders
- ★ `get_redstone_power` probing
- ○ Insign DSL: sign annotations → auto IO layout (`docs/insign-io-integration.md`)

## 6. Mesh & render
- ★ Meshing to GLB/glTF, USDZ, NUCM; vanilla resource packs
- ★ Headless GPU renderer; isometric, sphere-fit camera
- ○ Texture atlas, item models, chunk meshing as separate entry points

## 7. Versions & translation
- ★ DataConverter port (spottedleaf/PaperMC): cross-data-version migration + per-block loss reports
- ★ Java ↔ Bedrock via GeyserMC mappings (26.2 parity)

## 8. Scripting
- ★ Embedded Lua and JS engines against the full API

## 9. Storage
- ★ `StoreIo` (whole schematics by URI) + `Store` (raw KV)
- ★ Backends: mem, fs, S3, Redis, Postgres, callback

## 10. Bindings & platform
- ★ Diplomat single source of truth (`src/bridge/`) → 7 languages, committed + CI-diffed
- ★ Feature flags: `simulation`, `meshing`, `rendering`, `voxelize`, `autostack`, `scripting-{lua,js}`, `store-*`

## Gaps worth deciding on
`selection`, `DefinitionRegion`, `SchematicBuilder`/layer art, the CLI binary,
Insign, and the footprint/signature analysis tier are all real features with
little or no README presence.
