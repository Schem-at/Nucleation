# Shapes, brushes, and masked fills

## Build: shapes, brushes, palettes


Spheres, tori, cones, pyramids, and bezier ribbons, plus boolean combinators,
filled by brushes that pick each block:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shapes-gallery.png" width="700" alt="Shape gallery: sphere, torus, cone, pyramid, bezier ribbon">

A gradient brush follows a shape's own parameter, around the ring of a torus or
along a bezier, and snaps every color to a palette:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-torus.png" width="480" alt="Rainbow torus: a seamless curve gradient snapped to the wool palette">
</div>

```python
brush = Brush.curve_gradient(stops, rainbow_colors, InterpolationSpace.Oklab)
brush.set_palette(Palette.wool())
BuildingTool.fill(s, Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
```

The shaded brush lights a base color by surface normal, giving 3D-lit forms out
of flat blocks:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/shaded-sphere.png" width="300" alt="Lambertian-shaded terracotta sphere">
</div>

```python
brush = Brush.shaded(224, 130, 84,  -1.0, 0.7, -0.3)   # base color, light direction
brush.set_palette(Palette.terracotta())
BuildingTool.fill(s, Shape.sphere(0, 0, 0, 16), brush)
```

And palettes turn colors into blocks. Ask for pure white to pure black in 24
steps and the engine picks the blocks itself: distinct, ordered, with off-hue
candidates penalized (bottom row; above it, the lightness-sorted wool,
concrete, terracotta, and planks presets):

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/palette-ramps.png" width="740" alt="Preset palette ramps plus the engine-generated 24-step white-to-black ladder">

```python
Palette.grayscale().ramp_ids_json(255, 255, 255,  0, 0, 0,  24)
# 24 distinct blocks: white_wool ... iron_block ... deepslate_tiles ... black_concrete
```

Seven presets ship, each a curated set you can list with `block_ids_json`:
`concrete` and `wool` (16 dyed colors each), `terracotta` (17), `wood` (13 plank
tones), `grayscale` (81 neutrals), and the broad `decorative` (951) and
`structural` (316) sets. Between any two colors a palette interpolates directly:
`gradient_ids_json` samples the line evenly, `ramp_ids_json` picks distinct steps.

```python
Palette.concrete().gradient_ids_json(200, 30, 70,  245, 205, 55,  8)
# crimson -> gold: red, red, pink, orange, orange, yellow, yellow, yellow
Palette.terracotta().ramp_ids_json(220, 40, 44,  40, 80, 220,  6)   # distinct steps
# red, pink, magenta, purple, blue, light_blue terracotta
```

A gradient brush does the same in 3D and paints as it fills. Five pairs, each a
`linear_gradient` in Oklab over a dithered palette, so every ramp is a smooth
blend rather than a few hard steps:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-pairs.png" width="720" alt="Five smooth two-color block gradients: crimson to gold, teal to violet, lime to blue, magenta to cyan, orange to indigo">
</div>

Sweep a single hue around a closed loop instead of between two endpoints and the
ramp closes on itself with no seam. A trefoil knot, one hue dithered red -> blue
-> green -> red around its length:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/gradient-knot.png" width="440" alt="A trefoil knot colored by a seamless cyclic gradient sweeping red to blue to green and back">
</div>

And when a ramp still bands, dither it: `Palette.…().dithered()` makes every
brush alternate between the two nearest blocks per voxel (ordered Bayer,
deterministic). Hard bands on the left, dissolved on the right:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/dither-compare.png" width="740" alt="The same shaded sphere with hard palette snapping (banded) and dithered snapping (smooth)">

Or build palettes from pure color logic over the block database, no names, just
measured color values and block facts:

```python
b = PaletteBuilder.create()
b.chroma_below(0.022)               # near-neutral only
b.lightness_between(0.35, 0.75)     # mid-grays
b.full_blocks_only()
mid_grays = b.build()               # 40+ blocks, picked by math

Blocks.by_color_json(120, 200, 60, 0.10)
# everything lime-ish, nearest first: lime_concrete_powder (0.053), ...
```

And shapes aren't limited to the primitives: **any SDF tree is a `Shape`**, so
smooth-blended distance fields fill with every brush. Field-gradient normals
mean the shaded brush shades a blend continuously across the seam:

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/sdf-shape-shaded.png" width="400" alt="A smooth-union SDF blob filled with the shaded brush">
</div>

```python
blob = Shape.sdf('{"type": "smoothUnion", "k": 6.0, "a": {"type": "sphere", "radius": 10}, '
                 '"b": {"type": "translate", "offset": [11, 3, 0], "child": {"type": "sphere", "radius": 7}}}')
BuildingTool.fill(s, blob, shaded_brush)      # masked fills work too
```

More in the guides: [shapes & brushes](shapes-and-brushes.md) ·
[palettes, ramps, and pixel art](palettes-and-color.md).

## Edit without collateral damage


Masked fills touch only what you allow: `fill_only_air` builds around existing
work; `fill_replacing` swaps listed blocks inside a shape. Here a temple weathers
into moss and cracks within a sphere of decay:

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/masked-fill.png" width="760" alt="Greek temple before/after weathering via fill_replacing">

```python
BuildingTool.fill_replacing(temple, decay_sphere, weathered_brush,
                            '["minecraft:stone_bricks"]')
```

---

## Reference

The building tool fills a **shape** (which blocks) with a **brush** (which
block goes at each position). All of it is available in every binding.

## Shapes

`sphere`, `cuboid`, `ellipsoid`, `cylinder` / `cylinder_between`, `cone`,
`torus`, `pyramid`, `disk`, `plane`, `triangle`, `line`, `bezier`
(control points + thickness + resolution), and `polygon_prism` (a closed 2D
footprint extruded between two Y levels — building footprints, lake outlines)
— plus combinators `union_with`, `intersection_with`, `difference_with`, and
`hollow(thickness)`. An [SDF tree](sdf-and-fields.md) is a `Shape` too
(`Shape.sdf`), and so is a voxelized [mesh](meshing-and-rendering.md)
(`Voxelizer.shape_from_glb` / `shape_from_obj`).

Several shapes are **parametric**: a position inside them maps to a
parameter `t ∈ [0, 1]` (angle around a torus, distance along a line or
bezier, height up a cone/pyramid). Parametric brushes read it.

## Brushes

| Brush | What it does |
| --- | --- |
| `solid(block)` | one fixed block state |
| `color(r, g, b)` | nearest palette block to a color |
| `shaded(r, g, b, lx, ly, lz)` | Lambertian shading of a base color by surface normal, snapped to the palette |
| `linear_gradient(...)` | color gradient between two anchored points |
| `bilinear_gradient(...)` | four corner colors over a patch |
| `point_gradient(positions, colors, falloff, space)` | inverse-distance-weighted blend of colored anchor points |
| `curve_gradient(stops, colors, space)` | colors placed along a parametric shape's own `t` |

Every color brush takes `set_palette(palette)` and interpolates in `Rgb` or
`Oklab` (`InterpolationSpace`).

`curve_gradient` detail: `stops` are `t` values in `[0, 1]` with flat RGB
triples in `colors`. On a closed shape (torus), make the first and last
stop colors equal and the ring is seamless — the README's rainbow torus is:

```python
stops = [i / 6 for i in range(7)]
colors = [255, 40, 40,   255, 180, 0,   60, 200, 60,
          40, 180, 220,  60, 70, 230,   200, 60, 220,
          255, 40, 40]  # first == last -> seamless wrap
brush = Brush.curve_gradient(stops, bytes(colors), InterpolationSpace.Oklab)
brush.set_palette(Palette.wool())
BuildingTool.fill(s, Shape.torus(0, 0, 0, 16, 6, 0, 1, 0), brush)
```

## Masked fills

Fills that respect existing content:

```python
BuildingTool.fill(s, shape, brush)                     # overwrite everything
BuildingTool.fill_only_air(s, shape, brush)            # never touch placed blocks
BuildingTool.fill_replacing(s, shape, brush,
                            '["minecraft:stone_bricks"]')   # only listed blocks
```

The README's weathered temple is `fill_replacing` with a mossy/cracked
gradient brush inside a sphere — `scene_masked_fill` in
[`tools/readme-media/generate.py`](../../tools/readme-media/generate.py).

`rstack(s, shape, brush, count, dx, dy, dz)` stamps `count` offset copies.

Verified runnable examples: [`docs/readme-snippets/`](../readme-snippets/).
