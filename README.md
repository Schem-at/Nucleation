<div align="center">

# Nucleation

**A Minecraft schematic engine in Rust — load, build, simulate, mesh, and render
schematics from seven languages.**

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)
[![CI](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml/badge.svg)](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml)

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero.png" width="760" alt="A volcanic floating island generated from a JSON SDF description and rendered by nucleation">

*This volcano island is a JSON description — [signed distance fields](docs/guides/sdf-terrain.md)
plus material rules. Every image on this page was built **and rendered** by
nucleation itself, and every snippet ran for real
([images](tools/readme-media/generate.py) · [snippets + outputs](docs/readme-snippets/)).*

</div>

**Contents** · [Install](#install) · [The basics](#the-basics) ·
[Build](#build-shapes-brushes-palettes) · [Terrain](#terrain-from-a-json-description) ·
[Voxelize](#voxelize-3d-models) · [Redstone](#simulate-redstone) · [Mesh & render](#mesh-and-render) ·
[Analyze](#analyze-diff-fingerprint-auto-stack) · [Masked edits](#edit-without-collateral-damage) ·
[Worlds & block data](#worlds-versions-and-the-block-database) ·
[Languages](#one-api-seven-languages) · [Docs](#documentation--development)

## Install

```bash
cargo add nucleation        # Rust
npm  install nucleation     # JavaScript / TypeScript (Node ≥ 18 or a bundler)
pip  install nucleation     # Python (CPython 3.12+)
```

Kotlin/JVM, PHP, C, and C++ ship as archives on
[Releases](https://github.com/Schem-at/Nucleation/releases) —
[quickstarts below](#one-api-seven-languages).

## The basics

A `Schematic` is a named region of blocks (plus block entities, entities, and
metadata). Load one from any supported format, edit it with plain coordinates
and block strings, save it in any other:

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")   # any format, auto-detected
cube.dimensions()                                          # (3, 3, 3)

cube.set_block(1, 3, 1, "minecraft:glowstone")             # y=3: the region grows to fit
cube.get_block_name(1, 3, 1)                               # "minecraft:glowstone"

cube.save_to_file("cube.schem")                            # format from the extension
```

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/basics.png" width="380" alt="The cube from the snippet with its glowstone crown, rendered">
</div>

The same loop in JavaScript — the WASM build has no filesystem, so it's bytes
in, bytes out:

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const cube = Schematic.fromData(readFileSync("simple_cube.litematic"));
cube.setBlock(1, 3, 1, "minecraft:glowstone");
writeFileSync("simple_cube.schem", Buffer.from(cube.toSchematicB64(), "base64"));
```

Block-state strings with properties work anywhere a block is named —
`"minecraft:lever[face=floor,facing=east]"` — and every block string a
schematic can contain round-trips. Later Python snippets assume
`from nucleation import *` and an existing schematic `s`; each has a fully
runnable version with captured output in
[`docs/readme-snippets/`](docs/readme-snippets/).

## Build: shapes, brushes, palettes

Spheres, tori, cones, pyramids, bezier ribbons — plus boolean combinators —
filled by brushes that pick each block:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shapes-gallery.png" width="700" alt="Shape gallery: sphere, torus, cone, pyramid, bezier ribbon">

A gradient brush follows a shape's own parameter — around the ring of a
torus, along a bezier — and snaps every color to a palette:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-torus.png" width="480" alt="Rainbow torus: a seamless curve gradient snapped to the wool palette">
</div>

```python
brush = Brush.curve_gradient(stops, rainbow_colors, InterpolationSpace.Oklab)
brush.set_palette(Palette.wool())
BuildingTool.fill(s, Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
```

The shaded brush lights a base color by surface normal — 3D-lit forms out of
flat blocks:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shaded-sphere.png" width="300" alt="Lambertian-shaded terracotta sphere">
</div>

```python
brush = Brush.shaded(224, 130, 84,  -1.0, 0.7, -0.3)   # base color, light direction
brush.set_palette(Palette.terracotta())
BuildingTool.fill(s, Shape.sphere(0, 0, 0, 16), brush)
```

And palettes turn colors into blocks. Ask for pure white → pure black in 24
steps and the engine picks the blocks itself — distinct, ordered, off-hue
candidates penalized (bottom row; above it, the lightness-sorted wool,
concrete, terracotta, and planks presets):

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/palette-ramps.png" width="740" alt="Preset palette ramps plus the engine-generated 24-step white-to-black ladder">

```python
Palette.grayscale().ramp_ids_json(255, 255, 255,  0, 0, 0,  24)
# 24 distinct blocks: white_wool ... iron_block ... deepslate_tiles ... black_concrete
```

Or build palettes from pure color logic over the block database — no names,
just measured color values and block facts:

```python
b = PaletteBuilder.create()
b.chroma_below(0.022)               # near-neutral only
b.lightness_between(0.35, 0.75)     # mid-grays
b.full_blocks_only()
mid_grays = b.build()               # 40+ blocks, picked by math

Blocks.by_color_json(120, 200, 60, 0.10)
# everything lime-ish, nearest first: lime_concrete_powder (0.053), ...
```

And shapes aren't limited to the primitives — **any SDF tree is a `Shape`**,
so smooth-blended distance fields fill with every brush. Field-gradient
normals mean the shaded brush shades a blend continuously across the seam:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/sdf-shape-shaded.png" width="400" alt="A smooth-union SDF blob filled with the shaded brush">
</div>

```python
blob = Shape.sdf('{"type": "smoothUnion", "k": 6.0, "a": {"type": "sphere", "radius": 10}, '
                 '"b": {"type": "translate", "offset": [11, 3, 0], "child": {"type": "sphere", "radius": 7}}}')
BuildingTool.fill(s, blob, shaded_brush)      # masked fills work too
```

More in the guides: [shapes & brushes](docs/guides/shapes-and-brushes.md) ·
[palettes, ramps, and pixel art](docs/guides/palettes.md).

## Terrain from a JSON description

The same SDF trees that work as shapes scale up to whole terrains: sampled
through declarative material rules (surface shells, depth bands, gradients,
scatter) instead of a single brush. Deterministic: same JSON, same terrain,
every language.

```python
from nucleation import Sdf

island = '''{"type": "displace", "amplitude": 3, "frequency": 0.1, "seed": 7,
             "child": {"type": "ellipsoid", "radii": [14, 8, 14]}}'''
rules = '''{"fill": [
  {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
  {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
  {"block": "minecraft:stone"}]}'''

terrain = Sdf.schematic_from_sdf_auto(island, rules)
# → 29×18×29, 6,927 blocks
```

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/terrain-minimal.png" width="560" alt="The island the snippet above produces">
</div>

That's the minimal version; the volcano up top adds smooth-blended cones, a
cylinder-cored lava crater, and noise-gated snow. Smooth booleans even
animate into metaballs — recipes, node/rule schemas, and the gradient fill
rules live in the [SDF terrain guide](docs/guides/sdf-terrain.md).

## Voxelize 3D models

Real 3D models become schematics: GLB (with node transforms and embedded
textures) and OBJ load into a `MeshModel`, and a voxelized mesh is — like
everything else here — just a `Shape`. Inside/outside comes from
triangle-parity ray casting; normals come from the nearest triangle, so
lighting brushes simply work. The Utah teapot under one spotlight, through
the grayscale ladder:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/teapot-spotlight.png" width="640" alt="Voxelized Utah teapot lit by a single spotlight through a grayscale block palette">
</div>

```python
teapot = Voxelizer.shape_from_obj(teapot_obj, 56.0, 0.75)   # shell closes its thin ceramic walls
spot = Brush.spotlight(-38, 55, -52,  0.48, -0.54, 0.66,  46.0,  245, 242, 235)
spot.set_palette(gray_ramp)
BuildingTool.fill(s, teapot, spot)
```

And textures project onto the voxels: each block takes the palette-closest
color of its nearest surface point (barycentric UVs, bilinear sampling) —
the classic COLLADA duck, beak and eye catchlights intact:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/textured-duck.png" width="460" alt="The Khronos duck voxelized with its texture projected onto blocks">
</div>

```python
duck = Voxelizer.schematic_from_glb_textured(duck_glb, 44.0, 0.7, Palette.solid(), "duck")
# 25,641 blocks: yellow_wool body, orange beak, black eyes with snow-block catchlights
```

## Simulate redstone

Headless circuit simulation via [MCHPRS](https://github.com/MCHPR/MCHPRS)'s
redpiler — and it runs in the browser, since simulation ships in the WASM
build. Flip the lever, tick the world, and the lamp (and wire) light up:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/redstone.png" width="700" alt="Lever, wire and lamp before and after simulation">

```js
import { Schematic, MchprsWorld } from "nucleation";

const world = MchprsWorld.create(circuit);   // lever → wire → lamp
world.onUseBlock(0, 1, 0);                   // flip the lever
world.tick(2);
world.flush();
world.isLit(2, 1, 0);                        // → true
```

Beyond poking blocks: a typed executor drives circuits through named, typed
inputs and outputs (booleans, integers, floats, ASCII) with layout builders
for buses — see the [docs](docs/README.md).

## Mesh and render

Any schematic → GLB/glTF or USDZ using any vanilla-format resource pack,
plus the headless GPU renderer that drew this page. The sphere-fit camera
holds a rotation-stable frame — this turntable is 40 renders of the hero
island:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero-turntable.gif" width="440" alt="Rotation-stable turntable of the volcano island">
</div>

```python
mesh = MeshResult.create(schem, ResourcePack.from_bytes(pack_zip), MeshConfig.create())
glb = base64.b64decode(mesh.glb_data_b64())     # magic b'glTF'

cfg = RenderConfig.create(1200, 760)
cfg.set_isometric()
cfg.set_sphere_fit(True)                        # rotation-stable framing
Renderer.render_to_file(schem, pack_zip, cfg, "island.png")
```

## Analyze: diff, fingerprint, auto-stack

Structural diffs know what was added, removed, changed, and swapped — here
as a ghost view, additions in green, removals in red:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/diff-engine.png" width="760" alt="Cottage before, after, and diff ghost view">

```python
diff = Diff.compute(before, after, "exact")     # distance 3; summary JSON with regions
Fingerprint.is_duplicate(before, after, "exact")   # False (fingerprints are translation-invariant)
```

And nucleation can *find the repetition in a build* — the lattice of a
tiling wall, a repeater bus, a pixel grid — and restamp it to a new size:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/autostack.png" width="760" alt="A 2-unit wall module restamped to 6 units">

```python
Autostack.detect_structures(wall)        # {"mode": "1d", "vectors": [[4,0,0]], "coverage": 1.0}
longer = Autostack.resize_1d(wall, 4, 0, 0, 6)   # 2 units → 6: (8,4,1) → (24,4,1)
```

## Edit without collateral damage

Masked fills touch only what you allow: `fill_only_air` builds around
existing work; `fill_replacing` swaps listed blocks inside a shape — a
temple weathering into moss and cracks within a sphere of decay:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/masked-fill.png" width="760" alt="Greek temple before/after weathering via fill_replacing">

```python
BuildingTool.fill_replacing(temple, decay_sphere, weathered_brush,
                            '["minecraft:stone_bricks"]')
```

## Worlds, versions, and the block database

Schematics round-trip through *playable worlds* — export a real world folder
(`level.dat` + region files), import any world back, bounded or streamed
chunk-by-chunk in constant memory:

```python
plaza.save_world(world_dir, "")
back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, 39, 4, 39)
```

The built-in DataConverter port migrates blocks, items, and entities across
Minecraft data versions (loss reports on downgrades), and Java ↔ Bedrock
translation runs on GeyserMC's mappings at full **26.2** parity.

Under it all sits a block database extracted from Mojang's own data
generator and the vanilla jars — kinds, variant families, resolved tags,
geometry, measured colors for all 1,196 Minecraft 26.2 blocks — which
[updates itself](docs/guides/minecraft-block-data.md) when Mojang ships a
new version:

```python
json.loads(Blocks.get_json("minecraft:oak_stairs"))
# {"kind": "minecraft:stair", "base_block": "minecraft:oak_planks",
#  "tags": ["minecraft:mineable/axe", ...], "full_cube": false, ...}

json.loads(Blocks.variants_of_json("minecraft:oak_planks"))
# [oak_planks, oak_button, oak_fence, oak_fence_gate, oak_pressure_plate, oak_slab, ...]
```

## One API, seven languages

Every binding is generated from one annotated-Rust source of truth
([`src/bridge/`](src/bridge/)) via
[Diplomat](https://github.com/rust-diplomat/diplomat) — committed,
regenerated, and diffed in CI so they can never drift:

| Language | Package | Errors | Naming |
| --- | --- | --- | --- |
| Rust | `nucleation` crate (native API) | `Result` | `snake_case` |
| JavaScript | `npm install nucleation` | exceptions | `setBlock` |
| Python | `pip install nucleation` | exceptions | `set_block` |
| Kotlin/JVM | Release JAR (JNA, 5 platforms bundled) | `kotlin.Result` | `setBlock` |
| PHP | Release archive (`php/` + FFI) | `DiplomatError` | `setBlock` |
| C | Release archive (`include/` + library) | result structs | `Schematic_set_block` |
| C++ | Header-only over the C ABI | `diplomat::result` | `set_block` |

What ships where:

| Channel | Surface |
| --- | --- |
| npm | full surface; WASM includes simulation + meshing (no GPU rendering) |
| PyPI | full surface, including simulation, meshing, rendering, scripting |
| Release archives + JAR | full surface, native, 5 platforms |
| crates.io | full surface except `simulation`* |

\* MCHPRS isn't on crates.io — for simulation in Rust, use
`nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }`.

<details>
<summary><b>Kotlin, PHP, and C quickstarts</b></summary>

```kotlin
import at.schem.nucleation.*

val schematic = Schematic.create("demo")
schematic.setBlock(1, 2, 3, "minecraft:stone").getOrThrow()
println(schematic.getBlockName(1, 2, 3).getOrThrow()) // "minecraft:stone"
```

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

```c
#include "Schematic.h"

int main(void) {
    DiplomatStringView name = {"demo", 4};
    Schematic *s = Schematic_create(name);
    DiplomatStringView stone = {"minecraft:stone", 15};
    Schematic_set_block(s, 1, 2, 3, stone);
    Schematic_destroy(s);
    return 0;
}
```

</details>

## Documentation & development

- [Documentation index](docs/README.md) — per-language references and all
  feature guides ([shapes & brushes](docs/guides/shapes-and-brushes.md),
  [palettes](docs/guides/palettes.md), [SDF terrain](docs/guides/sdf-terrain.md),
  [scripting](docs/guides/scripting.md),
  [block database](docs/guides/minecraft-block-data.md))
- [`docs/readme-snippets/`](docs/readme-snippets/) — every snippet above with
  its verified output
- [Release notes](RELEASE_NOTES.md)

Also in the box: embedded Lua/JS scripting engines, pluggable storage
(memory / filesystem / S3 / Redis / Postgres behind one URI), and layer-art
templates (schematics from ASCII art).

```bash
cargo test                          # core suite (784 tests)
./tools/gen-bindings.sh             # regenerate bindings (diplomat-tool fork)
./examples/bridge_smoke/js/run.sh   # end-to-end smoke per language
```

CI regenerates bindings and fails on drift, exercises every built wheel and
the assembled JAR before release, and smoke-tests all seven language
bindings on every push.

## License

MIT. See [LICENSE](LICENSE).
