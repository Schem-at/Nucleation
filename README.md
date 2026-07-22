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
- [Shapes, brushes, masked fills](docs/features/shapes-and-brushes.md) — the building primitives
- [SDF shapes, terrain, and fields](docs/features/sdf-and-fields.md) — JSON-described geometry, terrain, Voronoi
- [Palettes and color](docs/features/palettes-and-color.md) — turning colors into blocks
- [Voxelizing 3D models](docs/features/voxelize.md) — GLB/OBJ meshes, texture projection
- [Geodata](docs/features/geo.md) — elevation grids and OSM footprints
- [Composition](docs/features/composition.md) — stacking the primitives
- [Regions, transforms, stamping](docs/features/regions-and-transforms.md) — multi-region builds and rigid motions
- [Chunk iteration, streaming, worlds](docs/features/streaming-and-worlds.md) — constant-memory pipelines and world I/O
- [Block entities, entities, NBT](docs/features/block-entities-nbt.md) — SNBT round-trips
- [Redstone simulation](docs/features/redstone-simulation.md) — MCHPRS redpiler, typed circuit executors
- [Meshing and rendering](docs/features/meshing-and-rendering.md) — GLB/glTF/USDZ and the headless renderer
- [Animating a build](docs/features/animation.md) — assembly, layer printing, reveals along a curve
- [Analysis](docs/features/analysis.md) — diff, fingerprint, auto-stack
- [Versions and translation](docs/features/versions-and-translation.md) — data-version migration, Java <-> Bedrock
- [The block database](docs/features/block-database.md) — 1,196 blocks, facts and measured colors
- [Embedded scripting](docs/features/scripting.md) — Lua and JS against the full API
- [Pluggable storage](docs/features/storage.md) — mem, fs, S3, Redis, Postgres
- [Bindings and languages](docs/features/bindings-and-languages.md) — one generated API, seven languages

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

```bash
cargo test                          # core suite
./tools/gen-bindings.sh             # regenerate bindings (diplomat-tool fork)
./examples/bridge_smoke/js/run.sh   # end-to-end smoke per language
```

CI regenerates bindings and fails on drift, exercises every built wheel and the
assembled JAR before release, and smoke-tests all seven language bindings on
every push.

## License

MIT. See [LICENSE](LICENSE).