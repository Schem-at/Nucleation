# Redstone simulation (JavaScript)

```js
import { Schematic, MchprsWorld } from "nucleation";

const circuit = Schematic.create("lamp_circuit");
for (let x = 0; x < 3; x++) circuit.setBlock(x, 0, 0, "minecraft:gray_concrete");
circuit.setBlockFromString(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
circuit.setBlockFromString(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
circuit.setBlockFromString(2, 1, 0, "minecraft:redstone_lamp[lit=false]");

const world = MchprsWorld.create(circuit);
console.log("lamp lit before:", Boolean(world.isLit(2, 1, 0)));
world.onUseBlock(0, 1, 0); // flip the lever
world.tick(2);
world.flush();
console.log("lamp lit after: ", Boolean(world.isLit(2, 1, 0)));
```

Output:

```text
lamp lit before: false
lamp lit after:  true
```

_Environment: Node v24.4.1 + nucleation 0.3.5 (npm, Diplomat WASM ES modules), macOS arm64._

<!-- `isLit` returns 0/1 (number) in JS rather than a boolean, hence the Boolean() wrap. -->
