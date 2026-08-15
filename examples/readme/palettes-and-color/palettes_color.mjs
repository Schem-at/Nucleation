import { writeFileSync } from "node:fs";
import { Palette, PaletteBuilder, Schematic } from "nucleation";


// --8<-- [start:choose]
const builder = PaletteBuilder.create();
builder.fullBlocksOnly();
builder.excludeTransparent();
builder.excludeFalling();
builder.survivalOnly();
builder.colorNear(42, 132, 92, 0.20);
const safeGreen = builder.build();

const concrete = Palette.concrete();
const gray = Palette.fromBlockIds(JSON.stringify([
  "minecraft:black_concrete",
  "minecraft:gray_concrete",
  "minecraft:light_gray_concrete",
  "minecraft:white_concrete",
]));
if (safeGreen.len() === 0 || concrete.len() !== 16 || gray.len() !== 4) {
  throw new Error("palette construction changed");
}
// --8<-- [end:choose]


// --8<-- [start:build]
const atlas = Schematic.create("color_atlas");

// A distinct 12-block ramp. No block id may repeat.
const ramp = JSON.parse(concrete.rampIdsJson(20, 50, 150, 250, 200, 30, 12));
for (let x = 0; x < 32; x++) {
  atlas.setBlock(x, 15, 0, ramp[Math.floor(x * ramp.length / 32)]);
}

// A 32-sample lookup table. Repeated ids are expected on a 16-color palette.
const gradient = JSON.parse(concrete.gradientIdsJson(20, 50, 150, 250, 200, 30, 32));
gradient.forEach((block, x) => atlas.setBlock(x, 13, 0, block));

// Ordered dithering extends a four-block grayscale palette across 32 values.
for (let y = 0; y < 12; y++) {
  for (let x = 0; x < 32; x++) {
    const value = Math.floor(x * 255 / 31);
    const block = gray.closestBlockDithered(value, value, value, x, y, 0);
    atlas.setBlock(x, y, 0, block);
  }
}
// --8<-- [end:build]


// --8<-- [start:inspect]
const size = atlas.tightDimensions();
if (atlas.blockCount() !== 448) throw new Error("block count changed");
if (`${size.x},${size.y},${size.z}` !== "32,16,1") throw new Error("size changed");
if (ramp.length !== 12 || new Set(ramp).size !== 12) throw new Error("ramp repeated");
if (gradient.length !== 32 || new Set(gradient).size === 32) throw new Error("gradient did not repeat");
// --8<-- [end:inspect]

const output = process.env.PALETTES_COLOR_OUT ?? "color-atlas.schem";
writeFileSync(output, Buffer.from(atlas.toSchematicB64(), "base64"));
console.log(`Palettes and color JavaScript example: OK (${output})`);
