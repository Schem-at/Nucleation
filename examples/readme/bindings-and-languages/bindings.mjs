import { writeFileSync } from "node:fs";
import { Schematic } from "nucleation";

// --8<-- [start:build]
const stack = Schematic.create("binding_stack");
stack.fillCuboid(-3, 0, -3, 3, 0, 3, "minecraft:polished_deepslate");
stack.fillCuboid(-2, 1, -2, 2, 1, 2, "minecraft:light_blue_concrete");
stack.fillCuboid(-1, 2, -1, 1, 2, 1, "minecraft:yellow_concrete");
stack.setBlock(0, 3, 0, "minecraft:emerald_block");

const size = stack.tightDimensions();
if (stack.blockCount() !== 84) throw new Error("block count changed");
if (`${size.x},${size.y},${size.z}` !== "7,4,7") throw new Error("bounds changed");
// --8<-- [end:build]


const bytes = Buffer.from(stack.toSchematicB64(), "base64");
const output = process.env.BINDINGS_OUT ?? "binding-stack.schem";
writeFileSync(output, bytes);
console.log(`Bindings JavaScript example: OK (${output})`);
