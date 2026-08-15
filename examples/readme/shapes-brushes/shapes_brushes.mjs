import { writeFileSync } from "node:fs";
import {
  Brush,
  BuildingTool,
  InterpolationSpace,
  Palette,
  Schematic,
  Shape,
} from "nucleation";


// --8<-- [start:build]
const garden = Schematic.create("orbital_garden");

// Shape chooses cells; Brush chooses the state written to those cells.
const plinth = Shape.cuboid(-20, 0, -16, 20, 2, 16);
BuildingTool.fill(garden, plinth, Brush.solid("minecraft:stone_bricks"));

// A mask limits the write to stone bricks already inside the sphere.
const weathering = Shape.sphere(-10, 2, 0, 8);
BuildingTool.fillReplacing(
  garden,
  weathering,
  Brush.solid("minecraft:mossy_stone_bricks"),
  '["minecraft:stone_bricks"]',
);

// A parametric torus supplies t in [0, 1] to a closed color gradient.
const stops = [0.0, 0.25, 0.5, 0.75, 1.0];
const colors = [255, 48, 48, 255, 190, 32, 64, 190, 255, 174, 72, 255, 255, 48, 48];
const orbit = Shape.torus(0, 14, 0, 12, 3, 0, 1, 0);
const rainbow = Brush.curveGradient(stops, colors, InterpolationSpace.Oklab);
rainbow.setPalette(Palette.wool());
BuildingTool.fill(garden, orbit, rainbow);

// Boolean composition produces one hollow shell from two overlapping spheres.
const shell = Shape.sphere(-4, 14, 0, 6)
  .unionWith(Shape.sphere(4, 14, 0, 6))
  .hollow(1);
const clay = Brush.shaded(224, 130, 84, -1.0, 0.7, -0.3);
clay.setPalette(Palette.terracotta());
BuildingTool.fill(garden, shell, clay);
// --8<-- [end:build]


// --8<-- [start:inspect]
const size = garden.tightDimensions();
console.log(garden.blockCount());
console.log([size.x, size.y, size.z]);
console.log(garden.getBlockString(-20, 0, -16));
// --8<-- [end:inspect]

if (garden.blockCount() !== 6_627) throw new Error("block count changed");
if (`${size.x},${size.y},${size.z}` !== "41,21,33") throw new Error("size changed");
if (garden.getBlockString(-20, 0, -16) !== "minecraft:stone_bricks") {
  throw new Error("plinth changed");
}

const bytes = Uint8Array.from(Buffer.from(garden.toSchematicB64(), "base64"));
const output = process.env.SHAPES_BRUSHES_OUT ?? "orbital-garden.schem";
writeFileSync(output, bytes);
console.log(`Shapes and brushes JavaScript example: OK (${output})`);
