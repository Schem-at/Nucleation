# Fields and patterns: Voronoi on color and geometry (Python)

The `cells` SDF node is Worley / Voronoi noise. One typed field drives a **field
brush** (color) and, through `eval_at`, **geometry** — the same pattern, both
ways.

```python
from nucleation import (Schematic, Shape, Brush, BuildingTool, Palette,
                        InterpolationSpace, Sdf, SdfCellMode)

field  = Sdf.cells(0.11, 7, 1.0, SdfCellMode.CellValue, 0.0)
stops  = [0.0, 0.25, 0.5, 0.75, 1.0]
colors = bytes([235,70,70, 240,175,45, 70,200,90, 60,150,235, 160,80,220])

# TEXTURE: color a sphere by which Voronoi cell each voxel falls in.
brush = Brush.field_sdf(field, stops, colors, 0.0, 1.0, InterpolationSpace.Oklab)
brush.set_palette(Palette.concrete())
mosaic = Schematic.create("mosaic")
BuildingTool.fill(mosaic, Shape.sphere(0, 0, 0, 20), brush)

# GEOMETRY: raise each column to its cell's value (a basalt terrain).
terrain = Schematic.create("terrain")
for x in range(40):
    for z in range(40):
        v = field.eval_at(float(x), 0.0, float(z))             # 0..1 per cell
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

_Environment: CPython 3.12.11 + nucleation 0.4.1 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- CellValue is per-cell constant (mosaic); F1 is distance to the nearest seed; F2 and F2MinusF1 expose the other Worley modes. Cells compose like every other Sdf: subtract an F2MinusF1 field to carve foam. Brush.field_sdf consumes the live graph; Brush.field remains the legacy JSON overload. -->
