# Shapes, brushes, and masked fills

The building tool fills a **shape** (which blocks) with a **brush** (which
block goes at each position). All of it is available in every binding.

## Shapes

`sphere`, `cuboid`, `ellipsoid`, `cylinder` / `cylinder_between`, `cone`,
`torus`, `pyramid`, `disk`, `plane`, `triangle`, `line`, `bezier`
(control points + thickness + resolution) — plus combinators
`union_with`, `intersection_with`, `difference_with`, and `hollow(thickness)`.

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
