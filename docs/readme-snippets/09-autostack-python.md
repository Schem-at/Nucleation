# Autostack: detect & restamp repetition (Python)

```python
import json
from nucleation import Schematic, Autostack

# A 4-wide window module -- sill, framed pane, lintel -- stamped 3x along x.
wall = Schematic.create("arcade")
for unit in range(3):
    x = unit * 4
    wall.fill_cuboid(x, 0, 0, x + 3, 0, 0, "minecraft:stone_bricks")  # sill
    wall.fill_cuboid(x, 1, 0, x + 3, 2, 0, "minecraft:oak_planks")    # frame
    wall.fill_cuboid(x + 1, 1, 0, x + 2, 2, 0, "minecraft:glass")     # pane
    wall.fill_cuboid(x, 3, 0, x + 3, 3, 0, "minecraft:oak_slab")      # lintel

for hit in json.loads(Autostack.detect_structures(wall)):
    print("mode:", hit["mode"], "vectors:", hit["vectors"], "coverage:", hit["coverage"])

d = wall.tight_dimensions()
print("before:", (d.x, d.y, d.z))
longer = Autostack.resize_1d(wall, 4, 0, 0, 8)  # restamp: 3 units -> 8
d = longer.tight_dimensions()
print("after: ", (d.x, d.y, d.z))
```

Output:

```text
mode: 1d vectors: [[4, 0, 0]] coverage: 1.0
before: (12, 4, 1)
after:  (32, 4, 1)
```

_Environment: CPython 3.14.6 + nucleation 0.3.6 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- Detection is sensitive to accidental vertical self-similarity: a symmetric plank/glass/glass/plank module reports a spurious `2d` mode ([[4,0,0],[0,2,0]], coverage 0.5) instead of the intended 1d hit. Giving each row a distinct block (sill/frame/lintel, as here) yields the clean `1d` + coverage 1.0. -->
