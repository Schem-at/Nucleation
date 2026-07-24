# SDF shapes, terrain, and fields

## Typed, composable fields

SDF authoring is object-based. Primitives, boolean operations, transforms, and
noise modifiers return immutable `Sdf` graphs; JSON is only an explicit
serialization and compatibility boundary.

```python
from nucleation import (
    Brush, BuildingTool, InterpolationSpace, Palette, Schematic, Sdf,
)

island = Sdf.ellipsoid(14, 8, 14).displace(
    amplitude=3,
    frequency=0.1,
    seed=7,
    octaves=3,
)

brush = Brush.linear_gradient(
    0, -8, 0, 45, 70, 170,
    0,  8, 0, 235, 190, 70,
    InterpolationSpace.Oklab,
)
brush.set_palette(Palette.concrete().dithered())

terrain = Schematic.create("island")
BuildingTool.fill(terrain, island.to_shape(), brush)
```

The graph can be evaluated (`eval_at`), queried for bounds and normals, reused
as geometry (`to_shape`), reused as color (`Brush.field_sdf`), and explicitly
serialized (`to_json`). `Sdf.from_json_string` imports old recipes. The legacy
`schematic_from_sdf*` entry points remain available for JSON material-rule
recipes, but are no longer the primary construction API.

Every bounded SDF conversion and sampler validates inclusive spans with widened
integer arithmetic and rejects work above 16,777,216 voxel centers before
iteration or allocation. Use tighter explicit bounds or split larger jobs.

<details>
<summary>Equivalent Java/Kotlin and JavaScript composition</summary>

Java uses the exception-based `SdfExpr` façade, avoiding Kotlin `Result` and
unsigned-type interop:

```java
var field = SdfExpr.sphere(12.0f)
    .subtract(SdfExpr.cappedCylinder(4.0f, 20.0f).rotate(90.0f, 0.0f, 0.0f))
    .smoothUnion(SdfExpr.sphere(5.0f).translate(10.0f, 0.0f, 0.0f), 2.0f);
```

JavaScript/TypeScript uses the generated `Sdf` directly:

```ts
const field = Sdf.sphere(12)
  .subtract(Sdf.cappedCylinder(4, 20).rotate(90, 0, 0))
  .smoothUnion(Sdf.sphere(5).translate(10, 0, 0), 2);
```

</details>

For Flow, use an `sdf` domain type and pass live WASM `Sdf` objects between
nodes. Primitive nodes output `Sdf`; operator/transform nodes consume and return
`Sdf`; `toShape` or a fill node is the terminal conversion. Flow persistence
may store `toJson()` output, but graph execution does not serialize between
nodes.

Slice the hero island in half and the material rules show their work: a grass
and dirt skin over a stone core that grades from deepslate at the roots up
through tuff to andesite, with the lava pool sitting in the crater.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/cross-section.png" width="720" alt="The volcano island sliced through the crater, exposing the lava pool and the stone strata inside">
</div>

Materials can also key on the **surface normal**, not just height and depth. On a
heightmap it is the gradient of the heights; on a solid build `DistanceField`'s
`slope` gives it directly. Its upward component decides the ground cover: gentle
ground greens over, steep faces stay rock, snow caps the flat peaks.

```python
ny = 2 / math.hypot(h[x+1] - h[x-1], h[z+1] - h[z-1], 2)   # upward normal: 1 flat, ->0 vertical
surf = "grass_block" if ny > 0.82 else "stone"             # + snow on high flats, scree on rock
```

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/slope-paint.png" width="760" alt="A rolling landscape painted by slope: grass on the flats, coarse dirt on the inclines, bare stone on the steepest faces, snow on the high flats">
</div>

## Fields and patterns


A pattern is a scalar field, and typed `Sdf` graphs represent both geometry and
standalone fields. The `cells` node adds Worley / Voronoi noise to that graph,
so one field stamps a pattern two ways. Point a **field brush** at it
to color by the field (each cell a flat color), or feed its value into
**geometry** (each cell's value drives a column's height):

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/voronoi-mosaic.png" width="330" alt="A sphere skinned with a Voronoi mosaic, each cell a flat color, from a field brush"> <img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/voronoi-columns.png" width="380" alt="A terrain of Voronoi cells raised to different heights, like basalt columns, from the same cells field">
</div>

```python
field = Sdf.cells(0.11, 7, 1.0, SdfCellMode.CellValue, 0.0)

# Texture: color every voxel by which Voronoi cell it falls in.
brush = Brush.field_sdf(field, stops, colors, 0.0, 1.0, InterpolationSpace.Oklab)
BuildingTool.fill(s, Shape.sphere(0, 0, 0, 28), brush)

# Geometry: raise each column to its cell's value.
for x, z in grid:
    h = field.eval_at(x, 0, z)                     # 0..1 per cell
    s.fill_cuboid(x, 0, z, x, round(1 + h * 20), z, block_for(h))
```

`cells` has `f1`, `f2`, and `f2MinusF1` (the classic crack field) modes too, and
it composes with every other SDF node: subtract it for a foam, intersect it,
warp it. Voronoi is one field; the same brush and the same node take any of the
others.

Put all three modes to work at once and you get a build, not a demo. This
fractured planet reads `f1` to shade each cell light at its center and dark at
its rim, cuts recessed buffer grooves along the `f2MinusF1` cracks, and pours a
glow down the surface normal. That last field needs no new tool: the depth into
the sphere is `R - length(p)`, exactly the signed distance an SDF returns, so the
same idea gives a gradient normal to *any* shape. Each groove wears a couple
layers of orange glass over light-emitting blocks that brighten with depth,
shroomlight fading into glowstone toward the core:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/voronoi-planet.png" width="460" alt="A black planet fractured into Voronoi cells, each lit brighter at its center, with glowing orange buffer cracks running between them">
</div>

```python
f1    = Sdf.cells(0.09, 4, 1.0, SdfCellMode.F1, 0.0)
crack = Sdf.cells(0.09, 4, 1.0, SdfCellMode.F2MinusF1, 0.0)
for x, y, z in inside_sphere(R):
    depth = R - length(x, y, z)                     # distance along the surface normal
    if depth > crust:                               # glowing core
        block = glow.snap(depth)                    # glass shell, then emitters deeper
    elif crack.eval_at(x, y, z) < crack_w:          # recessed buffer groove
        block = None if depth < inset else glow.snap(depth)
    else:                                            # cell crust
        block = cells.snap(shade(f1.eval_at(x, y, z)))     # light center, dark rim
```

None of that is sphere-specific: it is three fields over `(x, y, z)` plus a
depth. An SDF shape gets the depth for free (its own value), and for *any* other
build `DistanceField.from_schematic` runs the distance transform and hands back
the depth (and a surface normal). So the same material paints over arbitrary
geometry. Here it repaints the pre-existing hero island schematic block for
block, its glowing seams following the Voronoi field across the terrain:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/fracture-paint.png" width="620" alt="The hero volcano island repainted as a black fractured planet, glowing Voronoi crack seams running over its arch, peak, and floating shards">
</div>

The fractured look is not a built-in, just one rule written over those
primitives. Swap the rule and the same `DistanceField` naturalises instead: on a
clean stone-brick temple, slope and a patch-noise field settle moss and grass on
the flat tiers, creep mossy brick down the steps, and leave the steep walls bare
and cracked. Same primitives, ancient ruin:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/naturalise.png" width="760" alt="A clean stone-brick stepped temple beside the same temple naturalised: moss and grass on the flat tiers, mossy and cracked brick down the walls">
</div>

And those are two rules of many. The same handful of inputs, a `DistanceField`'s
depth and normal, block occupancy, and position, plus a field and a palette,
drive a whole range of treatments. Snow settles on up-facing surfaces; copper
greens with exposure; height bands a badlands mesa; ambient occlusion darkens the
recesses of a rock; and corners chip to mossy cobble by how many faces they
expose. None of it is a built-in, each is a short rule:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/treatments.png" width="820" alt="Five material treatments from the same primitives: snow accumulation, copper patina, badlands strata, ambient-occlusion cavity shading, and edge wear">
</div>

---

## Reference

Signed distance fields are immutable typed expression graphs. Identical graphs
yield identical values and schematics in every language. JSON import/export is
available for persistence and old recipes, not required for composition.

## Entry points

```python
field = Sdf.sphere(12).translate(0, 8, 0)
field = field.smooth_union(Sdf.torus(16, 4), 2)
value = field.eval_at(x, y, z)
normal = field.normal(x, y, z, 0.01)
shape = field.to_shape()                    # inferred finite bounds
shape = field.to_shape_bounded(*bounds)     # required for unbounded graphs
json_data = field.to_json()
field = Sdf.from_json_string(json_data)
```

Python additionally supports exact user functions as a bounded, synchronous
fill. The callable is not retained or serialized. Exceptions propagate and the
destination schematic is unchanged; omit `normal` for central differences:

```python
BuildingTool.fill_sdf_function(
    schematic, brush,
    -32, -16, -32, 31, 15, 31,
    lambda x, y, z: custom_distance(x, y, z),
    normal=lambda x, y, z: custom_gradient(x, y, z),
    epsilon=0.5,
)
```

The callback volume is capped at 16,777,216 voxels. Typed SDF graphs are the
portable path for Rust, JVM, JavaScript/WASM, and Python; callbacks are the
Python escape hatch for arbitrary runtime functions.

## Nodes

Primitives: `sphere`, `box_shape`, `torus`, `capsule`, `capped_cylinder`,
`capped_cone`, `ellipsoid`, `plane` (unbounded), `octahedron`, `hex_prism`,
`super_prism`, and `cells`. Operators: `union_with`, `intersection_with`,
`subtract`, `smooth_union`, `smooth_intersection`, `smooth_subtract`, `rounded`,
and `shell`. Transforms: `translate`, `rotate`, `scale`, `mirror`,
`repeat_infinite`, and `repeat_counted`. Noise modifiers: `displace` and `warp`.

Generated bindings adapt names conventionally: snake_case in Python,
camelCase in JavaScript/Kotlin/Java. `Brush.field_sdf` consumes a live typed
graph for scalar-field color; legacy `Brush.field` accepts JSON. The old
`Sdf.eval(json, ...)` and `Sdf.schematic_from_sdf*` APIs remain for
compatibility.

## Material rules

```json
{"fill": [
   {"when": {"depthBelowSurface": {"min": 0, "max": 0},
             "yRange": {"min": 14, "max": 64}},
    "block": "minecraft:snow_block"},
   {"when": {"depthBelowSurface": {"min": 1, "max": 3}}, "block": "minecraft:dirt"},
   {"gradient": {"palette": "grayscale", "from": [70, 68, 72],
                 "to": [150, 148, 152], "axis": "y", "range": [-14, 10]}}
 ],
 "surface": [
   {"density": 0.10, "on": "minecraft:grass_block",
    "blocks": ["minecraft:poppy", "minecraft:short_grass"]}
 ]}
```

First matching `fill` rule wins; a rule without `when` is the default.
Exactly one of `block` / `gradient` per rule. `gradient` palettes are the
preset names or `{"ids": [...]}`; `range` must be `[min, max]` (swap
`from`/`to` to invert the direction); `"ramp": "lightness"` indexes the
lightness-sorted palette directly instead of color-matching. `surface`
rules scatter decorations on matching surface blocks.

The [README](../../README.md)'s volcano island is one such tree —
`scene_hero` in [`tools/readme-media/generate.py`](../../tools/readme-media/generate.py)
is the full recipe (smooth-unioned ellipsoids and a cone, cylinder-cored
crater, 4-octave displacement, noise-gated snow, flower scatter).

## Metaballs

Smooth booleans animate into metaballs: move sphere centers each frame,
re-sample, render. This loop wears a white→black gradient of
survival-obtainable blocks painted by a single `gradient` fill rule:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/metaballs.gif" width="520" alt="Orbiting metaballs wearing a survival-block white-to-black gradient">
</div>

```python
rules = {"fill": [{"gradient": {
    "palette": {"ids": GRAY_RAMP},        # 19 survival blocks, snow -> black concrete
    "from": [8, 10, 14], "to": [250, 252, 252],
    "axis": "y", "range": [4, 17]}}]}
# one schematic_from_sdf call per frame, three spheres orbiting under smoothUnion k=10
```

The full scene — including the camera setup that keeps the loop from
pulsing (`RenderConfig.set_sphere_fit`) — is `scene_metaballs` in
[`tools/readme-media/generate.py`](../../tools/readme-media/generate.py).
You can also let the engine pick the ladder itself:
`Palette.grayscale().ramp_ids_json(255,255,255, 0,0,0, 19)`
([palette guide](palettes-and-color.md)).
