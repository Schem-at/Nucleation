// --8<-- [start:example]
import { readFileSync, writeFileSync } from "node:fs";
import { PNG } from "pngjs";
import { Schematic } from "nucleation";


function barrelPosition(x, y, channel) {
  const alternate = x & 1;
  const z = (channel & 1) === (y & 1) ? 5 + alternate : 5 * alternate;
  return [
    -((channel + y) & 1),
    -2 - channel - 3 * y,
    6 * Math.floor(x / 2) + z - 2,
  ];
}
const image = PNG.sync.read(readFileSync("rom-input.png"));
const rom = Schematic.create("image_rom");

for (let y = 0; y < image.height; y += 1) {
  for (let x = 0; x < image.width; x += 1) {
    const offset = (y * image.width + x) * 4;
    const [red, green, blue] = image.data.subarray(offset, offset + 3);

    for (const [channel, signal] of [blue >> 4, green >> 4, red >> 4].entries()) {
      rom.setBlock(
        ...barrelPosition(x, y, channel),
        `minecraft:barrel{signal=${signal}}`,
      );
    }
  }
}

const bytes = Buffer.from(rom.toSchematicB64(), "base64");
writeFileSync("image-rom.schem", bytes);
// --8<-- [end:example]


if (rom.blockCount() !== 16 * 10 * 3) throw new Error("block count changed");
console.log("Data-driven JavaScript example: OK");
