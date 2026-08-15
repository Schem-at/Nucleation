import { writeFileSync, mkdirSync } from "node:fs";
import { Schematic } from "nucleation";

// --8<-- [start:build]
const build = Schematic.create("round_trip");
build.fillCuboid(0, 0, 0, 3, 0, 3, "minecraft:stone_bricks");
build.setBlock(1, 1, 1, "minecraft:oak_stairs[facing=east,half=bottom]");
build.setBlock(2, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]");
build.setBlockWithNbt(
  0, 1, 0,
  "minecraft:chest[facing=south]",
  '{"CustomName":"Treasure"}',
);
// --8<-- [end:build]


// --8<-- [start:bytes]
const payload = Uint8Array.from(Buffer.from(build.saveAsB64("litematic", "", ""), "base64"));
const loaded = Schematic.fromData([...payload]); // content detection; no filename required
if (loaded.blockCount() !== 19) throw new Error("round trip changed");

const v3 = Buffer.from(loaded.saveAsB64("schematic", "v3", ""), "base64");
writeFileSync("round-trip.schem", v3);
// --8<-- [end:bytes]


const formats = new Map([
  ["litematic", ["", ".litematic"]],
  ["schematic", ["v3", ".schem"]],
  ["structure_snbt", ["", ".snbt"]],
  ["snapshot", ["", ".nusn"]],
  ["mcstructure", ["", ".mcstructure"]],
]);
const output = process.env.FORMATS_IO_OUT_DIR ?? "formats-output";
mkdirSync(output, { recursive: true });
for (const [formatName, [version, extension]] of formats) {
  const data = Buffer.from(build.saveAsB64(formatName, version, ""), "base64");
  const back = Schematic.fromData([...data]);
  if (back.blockCount() !== 19) throw new Error(`${formatName} changed`);
  writeFileSync(`${output}/round-trip${extension}`, data);
}

console.log(`Formats and I/O JavaScript example: OK (${output})`);
