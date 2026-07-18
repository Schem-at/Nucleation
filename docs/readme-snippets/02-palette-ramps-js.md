# Palette ramps (JavaScript)

```js
import { Palette, Schematic } from "nucleation";

const sunset = Palette.wool(); // all 16 wool colors, ready for gradient snapping
const ramp = JSON.parse(sunset.gradientIdsJson(255, 80, 40, 60, 40, 180, 8));

const strip = Schematic.create("sunset_strip");
ramp.forEach((wool, x) => strip.setBlock(x, 0, 0, wool));

console.log(ramp.join("\n"));
```

Output:

```text
minecraft:orange_wool
minecraft:red_wool
minecraft:red_wool
minecraft:red_wool
minecraft:magenta_wool
minecraft:purple_wool
minecraft:purple_wool
minecraft:blue_wool
```

_Environment: Node v24.4.1 + nucleation 0.3.5 (npm, Diplomat WASM ES modules), macOS arm64._
