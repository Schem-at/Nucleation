<div align="center">

# Nucleation

**A Minecraft schematic engine in Rust: load, build, simulate, mesh, and render
schematics from seven languages.**

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)
[![CI](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml/badge.svg)](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml)

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero.gif" width="760" alt="A scorched animated 3x7 torus knot whose braided geometry and raised cellular surface flow in a seamless loop">

*Every frame of this 3x7 knot is a separately generated schematic: its braid
advances while a periodic cellular field flows along the curve, cutting raised
scorched plates over a molten core. It was built **and rendered** by nucleation
([Python source](examples/readme/hero/) · [frame 0 `.schem`](docs/downloads/readme/hero/scorched-3x7-frame-000.schem) · [SDFs and fields](docs/features/sdf-and-fields.md)).*

</div>

## Install

```bash
cargo add nucleation        # Rust
npm  install nucleation     # JavaScript / TypeScript (Node ≥ 18 or a bundler)
pip  install nucleation     # Python (CPython 3.12+)
```

Kotlin/JVM, PHP, C, and C++ ship as archives on
[Releases](https://github.com/Schem-at/Nucleation/releases); see the
[quickstarts](docs/features/bindings-and-languages.md).

## Features

Every capability, with its own deep-dive doc:

- [Basics](docs/features/basics.md) — create, inspect, load, save, and download a complete example
- [Formats and I/O](docs/features/formats-and-io.md) — load, edit, and save every supported format
- [Regions, transforms, stamping](docs/features/regions-and-transforms.md) — deterministic multi-region builds, scoped rigid transforms, and reusable stamping
- [Shapes, brushes, masked fills](docs/features/shapes-and-brushes.md) — the building primitives
- [SDF shapes, terrain, and fields](docs/features/sdf-and-fields.md) — typed composable geometry, custom functions, terrain, Voronoi
- [Palettes and color](docs/features/palettes-and-color.md) — turning colors into blocks
- [Voxelizing 3D models](docs/features/voxelize.md) — GLB/OBJ meshes, texture projection
- [Geodata](docs/features/geo.md) — elevation grids and OSM footprints
- [Composition](docs/features/composition.md) — stacking the primitives
- [Chunk iteration, streaming, worlds](docs/features/streaming-and-worlds.md) — constant-memory pipelines and world I/O
- [World segmentation](docs/features/world-segmentation.md) — a whole world into individual builds with provenance, deterministically
- [Embedded schematic provenance](docs/features/schematic-provenance.md) — standardized world, map, dimension, coordinates, and source identity inside each build
- [Transformation and content policies](docs/features/transformation-policies.md) — canonical palettes, material standards, entity/NBT rules, UUID rewriting, and registry-safe inspection
- [Block entities, entities, NBT](docs/features/block-entities-nbt.md) — SNBT round-trips
- [Redstone simulation](docs/features/redstone-simulation.md) — MCHPRS redpiler, typed circuit executors
- [Tick simulation](docs/features/tick-simulation.md) — vanilla-accurate tick loop, verified against captures from the game

  <img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/tick-sim/subtick-wave.gif?v=2" width="470" alt="One game tick played in dispatch order: after a button press, the update cursor sweeps a 13x13 field of redstone dust in the engine's position-hash order">

- [Meshing and rendering](docs/features/meshing-and-rendering.md) — GLB/glTF/USDZ and the headless renderer
- [Animating a build](docs/features/animation.md) — assembly, layer printing, reveals along a curve
- [Analysis](docs/features/analysis.md) — diff, fingerprint, auto-stack
- [Versions and translation](docs/features/versions-and-translation.md) — data-version migration, Java <-> Bedrock
- [The block database](docs/features/block-database.md) — 1,196 blocks, facts and measured colors
- [Embedded scripting](docs/features/scripting.md) — Lua and JS against the full API
- [Pluggable storage](docs/features/storage.md) — mem, fs, SSH, S3, Redis, Postgres
- [Bindings and languages](docs/features/bindings-and-languages.md) — one generated API, seven languages

## Normalize imported schematics

Normalization is an explicit, versioned pipeline rather than an implicit part
of loading or saving. Dry-run the exact plan first, inspect its stable audit
report, then apply it atomically:

```python
from nucleation import Schematic, TransformPlan, inspect_transform, apply_transform

schematic = Schematic.open("incoming.schem")
plan = TransformPlan.registry_safe()

preview = inspect_transform(schematic, plan)  # never mutates schematic
if not preview.rejected and not preview.quarantined:
    report = apply_transform(schematic, plan)  # all passes commit, or none do
    schematic.save("normalized.schem")
```

Use `TransformPlan.canonical()` for lossless deterministic palette cleanup.
Use a custom plan for material conventions, text/NBT/item/entity rules, limits,
or UUID standardization. The complete [transformation-policy
guide](docs/features/transformation-policies.md) documents every field, default,
action, report, safety guarantee, Python helper, shared JSON contract, and the
storage-backed registry pipeline.

## The gallery

Ten more builds, each a short recipe that leans on the same handful of
primitives: a rainbow DNA helix and a trefoil knot, a Menger sponge, a fractal
tree, a gyroid, a Mandelbulb, a voxelized fox, a supershape, animated wave
interference, and type set in blocks.

<div align="center">
<a href="docs/gallery.md"><img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gallery-montage.png" width="900" alt="A gyroid, a trefoil knot, a voxelized fox, and a fractal tree, four of the gallery builds"></a>
</div>

Every one is a few dozen lines. Open the [gallery](docs/gallery.md) for all ten
with their code.

## Redstone EDA

Electronic design automation for redstone lives in
[`redstone-eda/`](redstone-eda/README.md): **Verilog in, exhaustively
sim-verified `.schem` out.**

<div align="center">
<a href="redstone-eda/README.md"><img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/redstone-eda/docs/img/kogge_stone_32bit.png" width="900" alt="A 32-bit Kogge-Stone prefix adder in redstone, 154,152 blocks, emitted by the HDL compiler"></a>
</div>

An HDL compiler (combinational *and* sequential — `always @(posedge clk)`
becomes a characterized DFF bank plus a clock spine), a verified
comparator-cell library, place-and-route (`crates/pnr-core` +
`crates/nucleation-routing`, with DRC/LVS/STA on a generated `Routing`
bridge), an interactive compositor with a
[browser app](apps/eda-studio/README.md), and 18 baked-at-rest artifacts
totalling 221,785 blocks and 2,660 verification checks — up to the 32-bit
Kogge-Stone adder above.

Nothing is trusted because it looks right: every artifact is proven in
[mc-tick](crates/mc-tick), the vanilla-accurate tick simulator, before it is
saved, and is saved settled so what you paste is what was proven.



## Documentation & development

- [Documentation index](docs/README.md): per-language references and all
  feature guides ([shapes & brushes](docs/features/shapes-and-brushes.md),
  [palettes](docs/features/palettes-and-color.md), [SDF terrain](docs/features/sdf-and-fields.md),
  [scripting](docs/features/scripting.md),
  [block database](docs/features/block-database.md))
- [`docs/readme-snippets/`](docs/readme-snippets/): every snippet in the
  feature docs, with its verified output
- [Release notes](RELEASE_NOTES.md)

Also in the box: layer-art templates (schematics from ASCII art).

**Start here: [`docs/DEV.md`](docs/DEV.md) — the iteration recipe.** The crate
builds under many feature combinations, and each one recompiles all 159
dependencies into its own artifacts; picking features ad hoc is what turned
edit-check loops into hour-long ones. `docs/DEV.md` has the canonical feature
set, the tier table, and the disk-hygiene rules.

```bash
brew install sccache                # prerequisite (wired in .cargo/config.toml)
tools/dev.sh fast [crate]           # the loop — seconds
tools/dev.sh pre-land               # tests + wasm32 + studio + smoke — minutes
tools/dev.sh full                   # the merge gate — exhaustive, all features
tools/doctor.sh                     # "why is my loop slow?" / disk hygiene

cargo test                          # core suite
./tools/gen-bindings.sh             # regenerate bindings (diplomat-tool fork)
./examples/bridge_smoke/js/run.sh   # end-to-end smoke per language
```

`.github/workflows/dev-tiers.yml` runs those same tiers on push and PR with
sccache and target caching. The release/manual CI matrix regenerates bindings
and fails on drift, exercises every built wheel and the assembled JAR, and
smoke-tests all seven language bindings.

## License

MIT. See [LICENSE](LICENSE).
