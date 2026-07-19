# SDF shapes and terrain

Signed distance fields describe geometry as JSON trees — primitives,
boolean/smooth operators, transforms, and seeded noise — which nucleation
samples into blocks through declarative material rules. Identical inputs
yield identical schematics in every language.

## Entry points

```python
Sdf.schematic_from_sdf_auto(sdf_json, rules_json)          # tree's own bounds
Sdf.schematic_from_sdf(sdf_json, rules_json, True, x0, y0, z0, x1, y1, z1)
Sdf.eval(sdf_json, x, y, z)                                # point query
```

## Nodes

Primitives: `sphere`, `box`, `torus`, `capsule`, `cappedCylinder`,
`cappedCone`, `ellipsoid`, `plane` (unbounded — needs explicit bounds),
`superPrism`. Operators: `union` (n-ary), `intersection`, `difference`,
`smoothUnion` / `smoothIntersection` / `smoothDifference` (with blend
radius `k`). Transforms: `translate`, `rotate`, `scale`. Noise:
`displace` (`amplitude`, `frequency`, `seed`, optional `octaves`) and `warp`;
`cells` for Worley / Voronoi (`frequency`, `seed`, `jitter`, `mode` one of
`f1` / `f2` / `f2MinusF1` / `value`, optional `threshold`). Any node also drives
color through `Brush.field`, and any point through `Sdf.eval`. Field names are
camelCase; see `src/sdf/node.rs` for the full schema.

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
([palette guide](palettes.md)).
