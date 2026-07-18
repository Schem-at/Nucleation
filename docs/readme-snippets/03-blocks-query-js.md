# Blocks query API (JavaScript)

```js
import { Blocks } from "nucleation";

const stairs = JSON.parse(Blocks.getJson("minecraft:oak_stairs"));
delete stairs.properties; // verbose: every state property and its values
console.log(JSON.stringify(stairs, null, 2));

console.log("oak_planks family:", JSON.parse(Blocks.variantsOfJson("minecraft:oak_planks")));
console.log("blocks tagged #wool:", JSON.parse(Blocks.byTagJson("wool")).length);
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
oak_planks family: [
  'minecraft:oak_planks',
  'minecraft:oak_button',
  'minecraft:oak_fence',
  'minecraft:oak_fence_gate',
  'minecraft:oak_pressure_plate',
  'minecraft:oak_slab',
  'minecraft:oak_stairs',
  'minecraft:petrified_oak_slab'
]
blocks tagged #wool: 16
```

_Environment: Node v24.4.1 + nucleation 0.3.5 (npm, Diplomat WASM ES modules), macOS arm64._
