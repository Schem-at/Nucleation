# Typed SDF terrain with a gradient (Python)

```python
from nucleation import (
    Brush, BuildingTool, InterpolationSpace, Palette, Schematic, Sdf,
)

field = Sdf.ellipsoid(14, 8, 14).displace(
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

terrain = Schematic.create("sdf-gradient")
BuildingTool.fill(terrain, field.to_shape(), brush)
terrain.save("sdf-gradient.litematic")

d = terrain.tight_dimensions()
print("terrain:", (d.x, d.y, d.z), "blocks:", terrain.block_count())
```

Output:

```text
terrain: (29, 18, 29) blocks: 6927
```

The complete runnable file is
[`examples/readme/sdf-gradient/generate.py`](https://github.com/Schem-at/Nucleation/blob/master/examples/readme/sdf-gradient/generate.py).
JSON is only needed when importing or exporting an old SDF recipe:
`Sdf.from_json_string(data)` / `field.to_json()`.
