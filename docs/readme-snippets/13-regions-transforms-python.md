# Regions, transforms & stamping (Python)

```python
import json
from nucleation import Schematic

# One schematic, many named regions (Litematica-style), addressed independently:
s = Schematic.create("build")
s.set_block_in_region("keep",  0, 0, 0, "minecraft:quartz_block")
s.set_block_in_region("gate", 10, 0, 0, "minecraft:blackstone")
print("regions:", s.region_names_json())
s.rotate_region_y("gate", 90)          # turn one region in place, leave the rest
print("rotated region 'gate' in place")

# Stamp a sub-volume of one schematic into another (target coords + exclude list):
bar = Schematic.create("bar")
bar.fill_cuboid(0, 0, 0, 9, 0, 0, "minecraft:stone")
bar.set_block(9, 0, 0, "minecraft:gold_block")
dst = Schematic.create("dst")
dst.copy_region(bar, 0, 0, 0,  9, 0, 0,  100, 0, 0,  "[]")
print("stamped blocks:", dst.block_count(), "| tip lands at 109:", dst.get_block_name(109, 0, 0))

# Whole-build transforms — rotate_x/y/z (degrees) and flip_x/y/z:
def gold(x):
    return next((b["x"], b["y"], b["z"]) for b in json.loads(x.get_all_blocks_json())
                if b["name"] == "minecraft:gold_block")
print("gold tip before rotate:", gold(bar))
bar.rotate_y(90)
print("gold tip after rotate_y(90):", gold(bar))
```

Output:

```text
regions: ["Main","keep","gate"]
rotated region 'gate' in place
stamped blocks: 10 | tip lands at 109: minecraft:gold_block
gold tip before rotate: (9, 0, 0)
gold tip after rotate_y(90): (0, 0, 0)
```

_Environment: CPython 3.14.6 + nucleation 0.3.10 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- Every new schematic starts with a default "Main" region; set_block_in_region creates named regions on demand. Per-region variants: rotate_region_x/y/z, flip_region_x/y/z, region_bounding_box_json, region_palette_json. -->
