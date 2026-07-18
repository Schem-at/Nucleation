# Shapes & brushes (Python)

```python
from nucleation import Schematic, Shape, Brush, Palette, BuildingTool, InterpolationSpace

orb = Schematic.create("orb")
dome = Shape.sphere(0, 0, 0, 10.0)
glow = Brush.linear_gradient(0, -10, 0,  40,  60, 200,   # bottom: deep blue
                             0,  10, 0, 250, 245, 235,   # top: warm white
                             InterpolationSpace.Oklab)
glow.set_palette(Palette.concrete())  # snap gradient colors to concrete blocks
BuildingTool.fill(orb, dome, glow)

d = orb.dimensions()
print("blocks:", orb.block_count(), "in", (d.x, d.y, d.z))
print("palette:", orb.palette_json())
```

Output:

```text
blocks: 4169 in (21, 21, 21)
palette: ["minecraft:air","minecraft:light_gray_concrete","minecraft:light_blue_concrete","minecraft:white_concrete","minecraft:blue_concrete"]
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64._
