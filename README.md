<div align="center">

# Nucleation

**A Minecraft schematic engine in Rust — load, build, simulate, mesh, and render
schematics from seven languages.**

[![Crates.io](https://img.shields.io/crates/v/nucleation.svg)](https://crates.io/crates/nucleation)
[![npm](https://img.shields.io/npm/v/nucleation.svg)](https://www.npmjs.com/package/nucleation)
[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)
[![CI](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml/badge.svg)](https://github.com/Schem-at/Nucleation/actions/workflows/ci.yml)

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero.png" width="760" alt="A volcanic floating island generated from a JSON SDF description and rendered by nucleation">

*This volcano island is a JSON description — signed distance fields plus material
rules. Every image on this page was built **and rendered** by nucleation
([`tools/readme-media/generate.py`](tools/readme-media/generate.py) regenerates them all).*

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

## The basics

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/basics.png" align="right" width="360" alt="A loaded schematic with a few edited blocks">

A `Schematic` is a named region of blocks (plus block entities, entities, and
metadata). Load one from any supported format — `.litematic`, Sponge `.schem`,
WorldEdit `.schematic`, Bedrock `.mcstructure`, structure `.nbt` — edit it with
plain coordinates and block strings, save it in any other:

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")
d = cube.dimensions()                  # (3, 3, 3)
cube.palette_json()                    # ["minecraft:air","minecraft:stone",...]

cube.set_block(1, 3, 1, "minecraft:glowstone")
cube.get_block_name(1, 3, 1)           # "minecraft:glowstone"
cube.set_block_from_string(0, 3, 1, "minecraft:lever[face=floor,facing=east]")

cube.save_to_file_with_format("simple_cube.schem", "", "")   # format from extension
```

The same loop in JavaScript — the WASM build has no filesystem, so it's bytes in,
bytes out:

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const cube = Schematic.fromData(readFileSync("simple_cube.litematic"));
cube.setBlock(1, 3, 1, "minecraft:glowstone");
writeFileSync("simple_cube.schem", Buffer.from(cube.toSchematicB64(), "base64"));
```

Every snippet on this page ran for real — the full set with captured outputs is
in [`docs/readme-snippets/`](docs/readme-snippets/).

## Sculpt with shapes and brushes

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-torus.png" align="right" width="380" alt="Rainbow torus">

Spheres, tori, cones, pyramids, bezier ribbons — plus boolean union / intersect /
subtract / hollow — filled by brushes. A curve gradient runs along a shape's own
parameter; wrap the first and last stops and the ring is seamless:

```python
stops = [i / 6 for i in range(7)]
colors = [255, 40, 40,   255, 180, 0,   60, 200, 60,
          40, 180, 220,  60, 70, 230,   200, 60, 220,
          255, 40, 40]  # first == last -> seamless wrap

brush = Brush.curve_gradient(stops, bytes(colors), InterpolationSpace.Oklab)
brush.set_palette(Palette.wool())
BuildingTool.fill(s, Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
```

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shapes-gallery.png" width="700" alt="Shape gallery: sphere, torus, cone, pyramid, bezier ribbon">

The shaded brush lights a base color by surface normal and snaps each shade to a
palette — instant 3D-lit forms out of flat blocks:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shaded-sphere.png" align="right" width="300" alt="Lambertian-shaded terracotta sphere">

```python
brush = Brush.shaded(224, 130, 84,   # base color
                     -1.0, 0.7, -0.3)  # light direction
brush.set_palette(Palette.terracotta())
BuildingTool.fill(s, Shape.sphere(0, 0, 0, 16), brush)
```

<br clear="right">

## Terrain from a JSON description

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/hero-turntable.gif" align="right" width="380" alt="Turntable of the volcano island">

Signed distance fields: primitives, smooth booleans, rotation, multi-octave
seeded noise — sampled into blocks through declarative material rules (depth
shells, Y bands, noise gates, gradient fills, surface scatter). The volcano up
top is exactly this, one JSON tree; here's the minimal version:

```python
from nucleation import Sdf

island = '''{"type": "displace", "amplitude": 3, "frequency": 0.1, "seed": 7,
             "child": {"type": "ellipsoid", "radii": [14, 8, 14]}}'''
rules = '''{"fill": [
  {"when": {"depthBelowSurface": {"min": 0, "max": 0}}, "block": "minecraft:grass_block"},
  {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
  {"block": "minecraft:stone"}]}'''

terrain = Sdf.schematic_from_sdf_auto(island, rules)
# → 29×18×29, 6,927 blocks — deterministic in every language
```

## Simulate redstone

Headless circuit simulation via [MCHPRS](https://github.com/MCHPR/MCHPRS)'s
redpiler. Flip the lever, tick the world, and the lamp — and the wire — light up:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/redstone.png" width="700" alt="Lever, wire and lamp before and after simulation">

```js
import { Schematic, MchprsWorld } from "nucleation";   // works in the browser too

const world = MchprsWorld.create(circuit);   // lever → wire → lamp
world.onUseBlock(0, 1, 0);                   // flip the lever
world.tick(2);
world.flush();
world.isLit(2, 1, 0);                        // → true
```

Beyond poking blocks: a typed executor drives circuits through named, typed
inputs/outputs (booleans, integers, floats, ASCII) with layout builders for
buses — see [`docs/`](docs/).

## Diff and fingerprint

Structural diffs know what was added, removed, changed, and swapped — here as a
ghost view, additions in green, removals in red:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/diff-engine.png" width="760" alt="Cottage before, after, and diff ghost view">

```python
diff = Diff.compute(before, after, "exact")
diff.distance()                                    # 3
diff.summary_json()                                # {"counts": {"added": 1, "removed": 2, ...}}
Fingerprint.is_duplicate(before, after, "exact")   # False — and fingerprints are
Fingerprint.compute(before, "exact")               # translation-invariant
```

## Auto-stack

Nucleation can *find the repetition in a build* — the lattice vectors of a
tiling wall, a bus of repeaters, a pixel grid — and restamp it to a new size:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/autostack.png" width="760" alt="A 2-unit wall module restamped to 6 units">

```python
Autostack.detect_structures(wall)      # {"mode": "1d", "vectors": [[4,0,0]], "coverage": 1.0}
longer = Autostack.resize_1d(wall, 4, 0, 0, 8)   # 3 units → 8: (12,4,1) → (32,4,1)
```

## Value → block: palettes

Palettes turn colors into blocks: presets (wool, concrete, terracotta, wood…),
tag/kind-filtered custom sets, lightness ramps, gradient sampling. Index a ramp
by any value — escape time, height, temperature:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/mandelbrot.png" width="380" alt="128x128 block mandelbrot">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/build-timelapse.gif" width="380" alt="The mandelbrot materializing">
</div>

```python
ramp = json.loads(Palette.wool().gradient_ids_json(255, 80, 40, 60, 40, 180, 8))
s.set_block(px, 0, pz, ramp[escape_iterations(px, pz)])
```

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/palette-ramps.png" width="700" alt="Lightness-sorted ramps: wool, concrete, terracotta, wood">

## Edit without collateral damage

Masked fills touch only what you allow: `fill_only_air` builds around existing
work; `fill_replacing` swaps listed blocks inside a shape — a temple weathering
into moss and cracks within a sphere of decay:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/masked-fill.png" width="760" alt="Greek temple before/after weathering via fill_replacing">

```python
BuildingTool.fill_replacing(temple, decay_sphere, weathered_brush,
                            '["minecraft:stone_bricks"]')
```

## Mesh and render

Any schematic → GLB/glTF or USDZ using any vanilla-format resource pack — with
packed texture atlases — plus the headless GPU renderer that drew this page:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/texture-atlas.png" align="right" width="240" alt="A packed texture atlas">

```python
mesh = MeshResult.create(schem, ResourcePack.from_bytes(pack_zip), MeshConfig.create())
glb = base64.b64decode(mesh.glb_data_b64())     # magic b'glTF'

cfg = RenderConfig.create(1200, 760)
cfg.set_isometric()
cfg.set_sphere_fit(True)                        # rotation-stable framing for turntables
Renderer.render_to_file(schem, pack_zip, cfg, "island.png")
```

<br clear="right">

## Script it

Embedded Lua and JavaScript engines generate schematics from scripts — with the
palette toolbox available inside the sandbox:

```lua
local wall = Schematic.new("sine_wall")
local ramp = palette_gradient_ids("concrete", 200, 60, 40, 255, 220, 80, 8)
for x = 0, 47 do
  local h = 6 + math.floor(5 * math.sin(x * math.pi / 12) + 0.5)
  for y = 0, h - 1 do wall:set_block(x, y, 0, ramp[y // 2 + 1]) end
end
result = wall
```

## Ask the block database

Facts for all 1,196 Minecraft **26.2** blocks, extracted from Mojang's own data
generator and the vanilla jars — kinds, variant families, resolved tags,
geometry, colors — and it [updates itself](docs/guides/minecraft-block-data.md)
when Mojang ships a new version:

```python
json.loads(Blocks.get_json("minecraft:oak_stairs"))
# {"kind": "minecraft:stair", "base_block": "minecraft:oak_planks",
#  "tags": ["minecraft:mineable/axe", "minecraft:stairs", ...],
#  "full_cube": false, "color": [162, 131, 79], "default_state": {...}}

json.loads(Blocks.variants_of_json("minecraft:oak_planks"))
# [oak_planks, oak_button, oak_fence, oak_fence_gate, oak_pressure_plate, oak_slab, ...]
```

## Worlds and versions

Schematics round-trip through *playable worlds*: export to a real world folder
(`level.dat` + region files), import any world back — bounded to a box or
streamed chunk-by-chunk in constant memory. And the built-in DataConverter port
migrates blocks, block entities, items, and entities across Minecraft data
versions, with loss reports on downgrades.

```python
plaza.save_world(world_dir, "")                            # a playable world
back = Schematic.from_world_directory_bounded(world_dir, 0, 0, 0, 39, 4, 39)
```

Java ↔ Bedrock translation (blockstates + block entities) runs on GeyserMC's
mappings at full 26.2 parity.

## One API, seven languages

Every binding is generated from one annotated-Rust source of truth
([`src/bridge/`](src/bridge/)) via [Diplomat](https://github.com/rust-diplomat/diplomat) —
committed, regenerated, and diffed in CI so they can never drift:

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
autostack, meshing, simulation, rendering, scripting). The crates.io crate has
meshing and rendering; `simulation` is git-only
(`nucleation = { git = "https://github.com/Schem-at/Nucleation", features = ["simulation"] }`)
because MCHPRS isn't on crates.io. The WASM build includes simulation and
meshing; GPU rendering is native-only.

## Documentation & development

- [Documentation index](docs/README.md) — per-language references and
  [feature guides](docs/guides/)
- [The Minecraft block database](docs/guides/minecraft-block-data.md) — data
  provenance and the weekly self-update workflow
- [`docs/readme-snippets/`](docs/readme-snippets/) — every snippet above with
  its verified output

```bash
cargo test                          # core suite
./tools/gen-bindings.sh             # regenerate bindings (diplomat-tool fork)
./examples/bridge_smoke/js/run.sh   # end-to-end smoke per language
```

CI regenerates bindings and fails on drift, exercises every built wheel and the
assembled JAR before release, and runs the language smoke tests on every push.

## License

MIT. See [LICENSE](LICENSE).
