# Regions, transforms & stamping (Python)

```python
import json
from nucleation import Schematic

build = Schematic.create("build")
build.set_block(0, 0, 0, "minecraft:quartz_block")
build.create_region("gate")
build.set_block_in_region(
    "gate", 10, 0, 0, "minecraft:oak_stairs[facing=east]"
)
print("regions:", build.region_names_json())

# Transform exactly one region. Main stays at the origin.
build.rotate_region_y("gate", 90)
gate = build.get_block_in_region("gate", 10, 0, 0)
print("gate facing:", json.loads(gate.properties_json())["facing"])
print("keep unchanged:", build.get_block_name(0, 0, 0))

# Stamp one named source region; its minimum corner maps to the target.
city = Schematic.create("city")
city.stamp_region(build, "gate", 50, 0, 20, "[]")
print("stamp:", city.get_block_name(50, 0, 20))

# Derive a rigid whole-schematic variant without mutating the source.
variant = build.deep_clone()
variant.rotate_schematic_y(90)
variant.translate_schematic(100, 0, 100)
print("variant keep:", variant.get_block_name(100, 0, 100))
print("original keep:", build.get_block_name(0, 0, 0))
```

Output:

```text
regions: ["Main","gate"]
gate facing: south
keep unchanged: minecraft:quartz_block
stamp: minecraft:oak_stairs
variant keep: minecraft:quartz_block
original keep: minecraft:quartz_block
```

_Environment: CPython 3.14.6 + locally built nucleation 0.3.18 wheel (`bridge-full`, cp312-abi3), macOS arm64._

<!-- The short rotate/flip/translate methods target Main. Explicit multi-region methods are rotate_schematic_x/y/z, flip_schematic_x/y/z, and translate_schematic. copy_region remains a compatibility alias for stamp_box. -->
