import { writeFileSync } from "node:fs";
import { Schematic } from "nucleation";

const WIDTH = 48;

function lightPositions() {
  const positions = new Map();
  for (let p = 0; p < WIDTH; p += 4) {
    for (const pos of [
      [p, 2, 0], [p, 2, WIDTH - 1], [0, 2, p],
      [WIDTH - 1, 2, p], [p, 2, WIDTH / 2], [WIDTH / 2, 2, p],
    ]) {
      positions.set(pos.join(","), pos);
    }
  }
  return [...positions.values()].sort((a, b) =>
    a[0] - b[0] || a[1] - b[1] || a[2] - b[2]
  );
}

function* towers() {
  for (let gx = 4; gx < 44; gx += 8) {
    for (let gz = 4; gz < 44; gz += 8) {
      yield [gx, gz, 6 + ((Math.floor(gx / 8) + Math.floor(gz / 8)) % 5) * 2];
    }
  }
}

// --8<-- [start:build]
const campus = Schematic.create("bulk_campus");

// A dense rectangular run belongs in the cuboid fast path.
campus.fillCuboid(
  0, 0, 0,
  WIDTH - 1, 1, WIDTH - 1,
  "minecraft:polished_deepslate",
);

// Sparse coordinates with one descriptor cross the WASM boundary once.
const lights = lightPositions().flat();
if (campus.setBlocks(lights, "minecraft:sea_lantern") !== 68) {
  throw new Error("light batch changed");
}

// Resolve the three tower materials once before the mixed-material hot loop.
const brick = campus.prepareBlock("minecraft:deepslate_bricks");
const glass = campus.prepareBlock("minecraft:light_blue_stained_glass");
const cap = campus.prepareBlock("minecraft:oxidized_cut_copper");

for (const [gx, gz, height] of towers()) {
  for (let y = 2; y < height + 2; y += 1) {
    const material = y === height + 1 ? cap : y % 3 === 0 ? glass : brick;
    for (let dx = 0; dx < 3; dx += 1) {
      for (let dz = 0; dz < 3; dz += 1) {
        campus.place(gx + dx, y, gz + dz, material);
      }
    }
  }
}
// --8<-- [end:build]


// --8<-- [start:inspect]
const size = campus.tightDimensions();
console.log(campus.blockCount());                     // 6926
console.log([size.x, size.y, size.z]);                 // [48, 16, 48]
console.log(campus.getBlockString(36, 15, 4));         // minecraft:oxidized_cut_copper
// --8<-- [end:inspect]

if (campus.blockCount() !== 6_926) throw new Error("block count changed");
if (`${size.x},${size.y},${size.z}` !== "48,16,48") throw new Error("size changed");
if (campus.getBlockString(36, 15, 4) !== "minecraft:oxidized_cut_copper") {
  throw new Error("tower cap changed");
}

const bytes = Uint8Array.from(Buffer.from(campus.toSchematicB64(), "base64"));
const output = process.env.FAST_GENERATION_OUT ?? "bulk-campus.schem";
writeFileSync(output, bytes);
console.log(`Fast generation JavaScript example: OK (${output})`);
