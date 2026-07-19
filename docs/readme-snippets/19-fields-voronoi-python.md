# Fields and patterns: Voronoi on color and geometry (Python)

The `cells` SDF node is Worley / Voronoi noise. One field JSON drives a **field
brush** (color) and, through `Sdf.eval`, **geometry** — the same pattern, both
ways.

```python
from nucleation import Schematic, Shape, Brush, BuildingTool, Palette, InterpolationSpace, Sdf

field  = '{"type": "cells", "frequency": 0.11, "seed": 7, "mode": "value"}'
stops  = [0.0, 0.25, 0.5, 0.75, 1.0]
colors = bytes([235,70,70, 240,175,45, 70,200,90, 60,150,235, 160,80,220])

# TEXTURE: color a sphere by which Voronoi cell each voxel falls in.
brush = Brush.field(field, stops, colors, 0.0, 1.0, InterpolationSpace.Oklab)
brush.set_palette(Palette.concrete())
mosaic = Schematic.create("mosaic")
BuildingTool.fill(mosaic, Shape.sphere(0, 0, 0, 20), brush)

# GEOMETRY: raise each column to its cell's value (a basalt terrain).
terrain = Schematic.create("terrain")
for x in range(40):
    for z in range(40):
        v = Sdf.eval(field, float(x), 0.0, float(z))          # 0..1 per cell
        terrain.fill_cuboid(x, 0, z, x, 1 + round(v * 16), z, "minecraft:stone")

md, td = mosaic.tight_dimensions(), terrain.tight_dimensions()
print("mosaic sphere:", (md.x, md.y, md.z), "blocks:", mosaic.block_count())
print("voronoi terrain:", (td.x, td.y, td.z), "blocks:", terrain.block_count())
```

Output:

```text
mosaic sphere: (41, 41, 41) blocks: 33401
voronoi terrain: (40, 18, 40) blocks: 15254
```

_Environment: CPython 3.14.6 + nucleation 0.3.17 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- cells modes: `value` (per-cell constant, mosaic), `f1` (distance to nearest seed), `f2`, `f2MinusF1` (the crack field). It is an SDF node like any other: `{"type":"subtract","a":{...shape...},"b":{"type":"cells","mode":"f2MinusF1","threshold":0.15}}` carves a foam. Brush.field takes any SDF JSON as the field, so an `fbm`/`displace` field paints marble and a coordinate expression paints stripes. -->
