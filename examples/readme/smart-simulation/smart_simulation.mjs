import { writeFileSync } from "node:fs";
import { Schematic, TickSettleMode, TickSimulation } from "nucleation";

// --8<-- [start:author]
const scene = Schematic.create("smart_circuit");
scene.fillCuboid(0, 0, 0, 8, 0, 2, "minecraft:smooth_stone");
scene.setBlock(0, 1, 0, "minecraft:lever[face=floor,facing=east,powered=false]");

// One engine setup, six placements. Each wire sees the state left by the last.
const wirePositions = Array.from({ length: 6 }, (_, i) => [i + 1, 1, 0]).flat();
if (scene.setBlocksSimulated(wirePositions, "minecraft:redstone_wire") !== 6) {
  throw new Error("wire batch changed");
}

scene.setBlock(7, 1, 0, "minecraft:redstone_lamp[lit=false]{simulate=true}");
scene.setBlock(0, 1, 2, "minecraft:barrel[facing=west]{signal=13,item=iron_ingot}");
// --8<-- [end:author]


// --8<-- [start:tick]
const tick = TickSimulation.fromSchematic(scene, TickSettleMode.InWorld, 0, 0, 0, "");
tick.useBlock(0, 1, 0);
tick.run(2);
if (tick.getBlock(7, 1, 0) !== "minecraft:redstone_lamp[lit=true]") {
  throw new Error("tick engine did not light lamp");
}
if (tick.tickCount() !== 2) throw new Error("tick count changed");
// --8<-- [end:tick]


if (scene.getBlockString(3, 1, 0) !==
    "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]") {
  throw new Error("wire state changed");
}
const barrel = JSON.parse(scene.getBlockEntityJson(0, 1, 2));
if (barrel.nbt.Items.List[0].Compound.id.String !== "minecraft:iron_ingot") {
  throw new Error("barrel shorthand changed");
}
if (scene.blockCount() !== 36) throw new Error("block count changed");

const bytes = Uint8Array.from(Buffer.from(scene.toSchematicB64(), "base64"));
const output = process.env.SMART_SIMULATION_OUT ?? "smart-circuit.schem";
writeFileSync(output, bytes);
console.log(`Smart simulation JavaScript example: OK (${output})`);
