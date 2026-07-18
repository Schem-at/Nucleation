<div align="center">

# Nucleation

**A Minecraft schematic engine in Rust — load, build, simulate, mesh, and render
schematics from seven languages.**

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)
[![CI](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml/badge.svg)](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml)

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero.png" width="760" alt="A floating island generated from a 20-line SDF JSON and rendered by nucleation">

*This island is a 20-line JSON description. Every image in this README was built
**and rendered** by nucleation itself — regenerate them all with
[`tools/readme-media/generate.py`](tools/readme-media/generate.py).*

</div>

## Install

```bash
cargo add nucleation        # Rust
npm  install nucleation     # JavaScript / TypeScript (Node ≥ 18 or a bundler)
pip  install nucleation     # Python (CPython 3.12+)
```

Kotlin/JVM, PHP, C, and C++ ship as archives on
[Releases](https://github.com/Schem-at/Nucleation/releases) —
[quickstarts below](#one-api-seven-languages).

## Thirty seconds

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")
d = cube.dimensions()
print((d.x, d.y, d.z))                        # (3, 3, 3)
print(cube.palette_json())                    # ["minecraft:air","minecraft:stone",...]

cube.set_block(1, 3, 1, "minecraft:glowstone")
cube.save_to_file_with_format("simple_cube.schem", "", "")   # format from extension
```

Same thing from JavaScript (the WASM build has no filesystem — bytes in, bytes out):

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const cube = Schematic.fromData(readFileSync("simple_cube.litematic"));
cube.setBlock(1, 3, 1, "minecraft:glowstone");
writeFileSync("simple_cube.schem", Buffer.from(cube.toSchematicB64(), "base64"));
```

Every snippet in this README is executed in CI-adjacent tooling with its real output
captured — the full set, with outputs, lives in
[`docs/readme-snippets/`](docs/readme-snippets/).

## Sculpt with shapes and brushes

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-torus.png" align="right" width="400" alt="Rainbow torus">

Spheres, tori, cones, pyramids, bezier ribbons — combined with boolean ops, filled by
brushes: solid, color-matched, gradient, or shaded. This torus is twelve rainbow
gradient points snapped to the wool palette:

```python
import colorsys, math
from nucleation import *

pos, cols = [], []
for i in range(12):
    a = 2 * math.pi * i / 12
    r, g, b = colorsys.hsv_to_rgb(i / 12, 0.95, 0.95)
    pos += [round(16 * math.cos(a)), 0, round(16 * math.sin(a))]
    cols += [int(r * 255), int(g * 255), int(b * 255)]

s = Schematic.create("torus")
brush = Brush.point_gradient(pos, bytes(cols), 4.0, InterpolationSpace.Oklab)
brush.set_palette(Palette.wool())
BuildingTool.fill(s, Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
```

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shapes-gallery.png" width="700" alt="Shape gallery: sphere, torus, cone, pyramid, bezier ribbon">

## Terrain from a JSON description

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero-turntable.gif" align="right" width="380" alt="Turntable of the SDF island">

Signed distance fields: primitives, smooth booleans, seeded noise — sampled into
blocks through declarative material rules (surface shells, depth bands, gradients,
flower scatter). Deterministic: same JSON, same island, every language.

```python
from nucleation import Sdf

island = '''{"type": "displace", "amplitude": 3, "frequency": 0.1, "seed": 7,
             "child": {"type": "ellipsoid", "radii": [14, 8, 14]}}'''
rules = '''{"fill": [
  {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
  {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
  {"block": "minecraft:stone"}]}'''

terrain = Sdf.schematic_from_sdf(island, rules, False, 0, 0, 0, 0, 0, 0)
# → 29×18×29, 6,927 blocks
```

## Paint with palettes

Palettes turn *colors* into *blocks*: presets (wool, concrete, terracotta, wood,
solid, …), tag/kind-filtered custom sets, lightness-sorted ramps, and gradient
sampling — the value→block workflow behind heatmaps, fractals, and pixel art.

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/palette-ramps.png" width="700" alt="Lightness-sorted ramps: wool, concrete, terracotta, wood">

```python
from nucleation import Palette
import json

ramp = json.loads(Palette.wool().gradient_ids_json(255, 80, 40, 60, 40, 180, 8))
# → orange_wool, red_wool, red_wool, red_wool, magenta_wool, purple_wool, purple_wool, blue_wool
```

Index a ramp by any value you like — say, Mandelbrot escape time:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mandelbrot.png" width="420" alt="128x128 block mandelbrot">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/build-timelapse.gif" width="420" alt="The mandelbrot materializing block by block">
</div>

```python
for pz in range(128):
    for px in range(128):
        s.set_block(px, 0, pz, ramp[escape_iterations(px, pz)])
```

## Edit without collateral damage

Masked fills touch only what you tell them to: `fill_only_air` builds around
existing work, `fill_replacing` swaps listed blocks inside a shape — here, stone
bricks weathering into mossy and cracked variants within a sphere:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/masked-fill.png" width="760" alt="Castle before/after fill_replacing">

```python
BuildingTool.fill_replacing(
    castle, Shape.sphere(0, 4, 12, 9.0), weathered_brush,
    '["minecraft:stone_bricks"]')
```

## Ask the block database

Block facts for all 1,196 Minecraft **26.2** blocks, extracted from Mojang's own
data generator and the vanilla jars — kinds, base-block variant links, resolved
tags, model-derived geometry, texture-derived colors:

```python
from nucleation import Blocks
import json

json.loads(Blocks.get_json("minecraft:oak_stairs"))
# {"kind": "minecraft:stair", "base_block": "minecraft:oak_planks",
#  "tags": ["minecraft:mineable/axe", "minecraft:stairs", "minecraft:wooden_stairs"],
#  "full_cube": false, "color": [162, 131, 79],
#  "default_state": {"facing": "north", "half": "bottom", ...}, ...}

json.loads(Blocks.variants_of_json("minecraft:oak_planks"))
# [oak_planks, oak_button, oak_fence, oak_fence_gate, oak_pressure_plate, oak_slab, oak_stairs, ...]
```

The data refreshes itself: a weekly workflow watches Mojang's version manifest and
opens a PR with regenerated data when a new Minecraft ships
([how it works](docs/guides/minecraft-block-data.md)).

## Simulate redstone

Headless circuit simulation via [MCHPRS](https://github.com/MCHPR/MCHPRS)'s
redpiler — flip levers, tick, read lamps; or drive whole circuits through a typed
executor with named inputs and outputs. Works in the browser (WASM) too:

```js
import { Schematic, MchprsWorld } from "nucleation";

const circuit = Schematic.create("lamp_circuit");
for (let x = 0; x < 3; x++) circuit.setBlock(x, 0, 0, "minecraft:gray_concrete");
circuit.setBlockFromString(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
circuit.setBlockFromString(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
circuit.setBlockFromString(2, 1, 0, "minecraft:redstone_lamp[lit=false]");

const world = MchprsWorld.create(circuit);
world.onUseBlock(0, 1, 0);   // flip the lever
world.tick(2);
world.flush();
world.isLit(2, 1, 0);        // → true
```

## Diff, fingerprint, deduplicate

Structural diffs (added / removed / changed / swapped, translation-aware) and
translation-invariant fingerprints for duplicate detection:

```python
diff = Diff.compute(before, after, "exact")
diff.distance()                              # 1
Fingerprint.is_duplicate(before, after, "exact")   # False
Fingerprint.compute(before, "exact")         # "3fdae2c9855e4794b30f9895b0d31a2c"
```

## Mesh and render

Any schematic → GLB/glTF or USDZ using any vanilla-format resource pack, and a
headless GPU renderer for PNG previews — the one that drew every image on this
page:

```python
mesh = MeshResult.create(schem, ResourcePack.from_bytes(pack_zip), MeshConfig.create())
glb = base64.b64decode(mesh.glb_data_b64())   # 9,848 bytes, magic b'glTF'

cfg = RenderConfig.create(1200, 760)
cfg.set_isometric()
Renderer.render_to_file(schem, pack_zip, cfg, "island.png")
```

## Formats and worlds

| | |
|---|---|
| **Schematics** | `.litematic` · Sponge `.schem` · WorldEdit `.schematic` · Bedrock `.mcstructure` · structure `.nbt` · `.nusn` (fast binary snapshot) — with auto-detection |
| **Worlds** | import Anvil region files / world folders (optionally bounded), export schematics as playable worlds, stream chunk-by-chunk in constant memory |
| **Versions** | convert blocks, block entities, items, and entities across Minecraft data versions (a Rust port of PaperMC's DataConverter), with loss reports |
| **Bedrock** | Java ↔ Bedrock blockstate + block-entity translation via GeyserMC mappings (full 26.2 parity) |

And more: **auto-stack** (detect a build's repeating lattice and restamp it bigger — a
4-bit adder into an 8-bit one), **embedded scripting** (generate schematics from Lua
or JS scripts, palettes included), **pluggable storage** (memory / filesystem / S3 /
Redis / Postgres behind one URI), and **layer-art templates** (schematics from ASCII
art).

## One API, seven languages

Every binding is generated from one annotated-Rust source of truth
([`src/bridge/`](src/bridge/)) via [Diplomat](https://github.com/rust-diplomat/diplomat) —
committed, regenerated, and diffed in CI so they can never drift. Same types, same
methods, per-language idioms:

| Language | Package | Errors | Naming |
| --- | --- | --- | --- |
| Rust | `nucleation` crate (native API) | `Result` | `snake_case` |
| JavaScript | `npm install nucleation` | exceptions | `setBlock` |
| Python | `pip install nucleation` | exceptions | `set_block` |
| Kotlin/JVM | Release JAR (JNA, 5 platforms bundled) | `kotlin.Result` | `setBlock` |
| PHP | Release archive (`php/` + FFI) | `DiplomatError` | `setBlock` |
| C | Release archive (`include/` + library) | result structs | `Schematic_set_block` |
| C++ | Header-only over the C ABI | `diplomat::result` | `set_block` |

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

**What ships where:** npm, PyPI, the release archives, and the JAR carry the full
surface (schematics, formats, worlds, building, palettes, SDF, diff/fingerprint,
meshing, simulation, rendering). The crates.io crate has meshing and rendering;
`simulation` is git-only (`nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }`)
because MCHPRS isn't on crates.io. The WASM build includes simulation and meshing;
GPU rendering is native-only.

## Documentation & development

- [Documentation index](docs/README.md) — per-language references and
  [feature guides](docs/guides/)
- [The Minecraft block database](docs/guides/minecraft-block-data.md) — where the
  data comes from and how it self-updates
- [`docs/readme-snippets/`](docs/readme-snippets/) — every README snippet with its
  verified output

```bash
cargo test                          # core suite
./tools/gen-bindings.sh             # regenerate bindings (diplomat-tool fork)
./examples/bridge_smoke/js/run.sh   # end-to-end smoke per language
```

CI regenerates bindings and fails on drift, exercises every built wheel and the
assembled JAR before release, and runs the language smoke tests on every push.

## License

MIT. See [LICENSE](LICENSE).
