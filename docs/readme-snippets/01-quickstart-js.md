# Quickstart (JavaScript)

```js
import { Schematic } from "nucleation";
import { readFileSync, writeFileSync } from "node:fs";

const cube = Schematic.fromData(readFileSync("simple_cube.litematic"));
const d = cube.dimensions();
console.log("dimensions:", [d.x, d.y, d.z]);
console.log("palette:   ", cube.paletteJson());

cube.setBlock(1, 3, 1, "minecraft:glowstone"); // crown the cube
console.log("new top:   ", cube.getBlockName(1, 3, 1));

writeFileSync("simple_cube.schem", Buffer.from(cube.toSchematicB64(), "base64"));
console.log("saved simple_cube.schem");
```

Output:

```text
dimensions: [ 3, 3, 3 ]
palette:    ["minecraft:air","minecraft:stone","minecraft:dirt","minecraft:oak_planks"]
new top:    minecraft:glowstone
saved simple_cube.schem
```

_Environment: Node v24.4.1 + nucleation 0.3.5 (npm, Diplomat WASM ES modules), macOS arm64, run next to `simple_cube.litematic`._

<!-- `Schematic.loadFromFile` / `saveToFile` exist in the npm typings but throw NucleationError.Io under Node (the WASM build has no filesystem) — go through `fromData(bytes)` and the `to*B64()` exporters instead. -->
