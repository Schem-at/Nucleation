# Blocks query API (Python)

```python
import json
from nucleation import Blocks

stairs = json.loads(Blocks.get_json("minecraft:oak_stairs"))
del stairs["properties"]  # verbose: every state property and its values
print(json.dumps(stairs, indent=2))

print("oak_planks family:", json.loads(Blocks.variants_of_json("minecraft:oak_planks")))
print("blocks tagged #wool:", len(json.loads(Blocks.by_tag_json("wool"))))
```

Output:

```text
{
  "base_block": "minecraft:oak_planks",
  "color": [
    162,
    131,
    79
  ],
  "default_state": {
    "facing": "north",
    "half": "bottom",
    "shape": "straight",
    "waterlogged": "false"
  },
  "emit_light": 0,
  "full_cube": false,
  "has_block_entity": false,
  "id": "minecraft:oak_stairs",
  "kind": "minecraft:stair",
  "tags": [
    "minecraft:mineable/axe",
    "minecraft:stairs",
    "minecraft:wooden_stairs"
  ],
  "transparent": false
}
oak_planks family: ['minecraft:oak_planks', 'minecraft:oak_button', 'minecraft:oak_fence', 'minecraft:oak_fence_gate', 'minecraft:oak_pressure_plate', 'minecraft:oak_slab', 'minecraft:oak_stairs', 'minecraft:petrified_oak_slab']
blocks tagged #wool: 16
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- The deleted `properties` key holds the full map of state properties to their allowed values; it is real but too long to show inline. -->
