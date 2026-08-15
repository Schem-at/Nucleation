// --8<-- [start:beacon]
import { readFileSync, writeFileSync } from "node:fs";
import { Schematic } from "nucleation";

function bytesFromBase64(value) {
  return Uint8Array.from(Buffer.from(value, "base64"));
}

const beacon = Schematic.create("beacon");
for (let x = -1; x <= 1; x += 1) {
  for (let z = -1; z <= 1; z += 1) {
    beacon.setBlock(x, 0, z, "minecraft:gold_block");
  }
}
beacon.setBlock(0, 1, 0, "minecraft:beacon");

const beaconBytes = bytesFromBase64(beacon.toSchematicB64());
writeFileSync("beacon.schem", beaconBytes);
// --8<-- [end:beacon]

if (beacon.blockCount() !== 10) throw new Error("beacon block count changed");
if (beacon.tightDimensions().x !== 3) throw new Error("beacon width changed");


// --8<-- [start:crafting-nook]
const nook = Schematic.create("crafting_nook");
for (let x = 0; x < 5; x += 1) {
  for (let z = 0; z < 5; z += 1) {
    nook.setBlock(x, 0, z, "minecraft:spruce_planks");
  }
}

function wallBlock(i, y, endPosts) {
  if (i === 2 && y === 2) return "minecraft:light_blue_stained_glass";
  if (endPosts.includes(i)) return "minecraft:stripped_spruce_log[axis=y]";
  return "minecraft:oak_planks";
}

for (const y of [1, 2, 3]) {
  for (let x = 0; x < 5; x += 1) {
    nook.setBlock(x, y, 0, wallBlock(x, y, [0, 4]));
  }
  for (let z = 1; z < 5; z += 1) {
    nook.setBlock(0, y, z, wallBlock(z, y, [4]));
  }
}

nook.setBlock(1, 1, 1, "minecraft:crafting_table");
nook.setBlock(3, 1, 1, "minecraft:chest[facing=south]");
nook.setBlock(4, 2, 1, "minecraft:wall_torch[facing=south]");
nook.setBlock(1, 2, 4, "minecraft:wall_torch[facing=east]");
writeFileSync(
  "crafting-nook.schem",
  bytesFromBase64(nook.toSchematicB64()),
);
// --8<-- [end:crafting-nook]

if (nook.blockCount() !== 56) throw new Error("crafting nook block count changed");


// --8<-- [start:coordinates]
const build = Schematic.create("signed_coordinates");
build.setBlock(-8, 64, 12, "minecraft:stone");
build.setBlock(24, 80, -3, "minecraft:glass");

const minimum = build.tightBoundsMin();
const maximum = build.tightBoundsMax();
const size = build.tightDimensions();
console.log([minimum.x, minimum.y, minimum.z]); // [-8, 64, -3]
console.log([maximum.x, maximum.y, maximum.z]); // [24, 80, 12]
console.log([size.x, size.y, size.z]);          // [33, 17, 16]
// --8<-- [end:coordinates]

if (`${minimum.x},${minimum.y},${minimum.z}` !== "-8,64,-3") {
  throw new Error("minimum bounds changed");
}
if (`${maximum.x},${maximum.y},${maximum.z}` !== "24,80,12") {
  throw new Error("maximum bounds changed");
}
if (`${size.x},${size.y},${size.z}` !== "33,17,16") {
  throw new Error("tight dimensions changed");
}


// --8<-- [start:block-states]
const inspect = Schematic.create("inspect");
inspect.setBlock(1, 1, 1, "minecraft:oak_log[axis=x]");
console.log(inspect.getBlockName(1, 1, 1));   // minecraft:oak_log
console.log(inspect.getBlockString(1, 1, 1)); // minecraft:oak_log[axis=x]

inspect.setBlock(1, 1, 1, "minecraft:air");  // remove it
// --8<-- [end:block-states]

if (inspect.blockCount() !== 0) throw new Error("air did not remove the block");


// --8<-- [start:contents]
const contents = Schematic.create("contents");
contents.setBlock(0, 0, 0, "minecraft:barrel{signal=13,item=diamond}");
contents.setBlock(1, 0, 0, "minecraft:chest{items=[diamond*64,emerald*12]}");
contents.setBlock(2, 0, 0, "minecraft:jukebox{record=pigstep}");
contents.setBlock(3, 0, 0, "minecraft:jukebox{signal=13}");
// --8<-- [end:contents]

if (contents.blockCount() !== 4) throw new Error("content shorthand placement failed");


// --8<-- [start:simulation]
const circuit = Schematic.create("placed_by_engine");
circuit.setBlock(4, 0, 0, "minecraft:redstone_block");
circuit.setBlock(5, 0, 0, "minecraft:redstone_wire{simulate=true}");
console.log(circuit.getBlockString(5, 0, 0));
// minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]
// --8<-- [end:simulation]

if (
  circuit.getBlockString(5, 0, 0) !==
  "minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]"
) {
  throw new Error("simulated wire state changed");
}


// --8<-- [start:io]
const copy = Schematic.fromData(readFileSync("beacon.schem"));
copy.setBlock(0, 2, 0, "minecraft:glass");
const editedBytes = bytesFromBase64(copy.toLitematicB64());
writeFileSync("beacon-edited.litematic", editedBytes);
// --8<-- [end:io]

if (editedBytes.length === 0 || copy.blockCount() !== 11) {
  throw new Error("file round trip failed");
}
console.log("Basics JavaScript examples: OK");
