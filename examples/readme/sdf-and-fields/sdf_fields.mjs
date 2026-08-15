import { writeFileSync } from "node:fs";
import {
  Brush,
  BuildingTool,
  Field3,
  InterpolationSpace,
  Palette,
  Schematic,
  Sdf,
} from "nucleation";


// --8<-- [start:graph]
const field = Field3.valueNoiseFbm(0.13, 73, 3);

const body = Sdf.ellipsoid(11, 7, 11).offsetByField(field, 1.7);
const shaft = Sdf.cappedCylinder(3.2, 12);
const equator = Sdf.torus(9.2, 1.45);
const form = body.subtract(shaft).smoothUnion(equator, 0.7);
// --8<-- [end:graph]


// --8<-- [start:build]
const brush = Brush.field3(
  field,
  [0.0, 0.5, 1.0],
  [25, 38, 105, 42, 185, 165, 245, 185, 48],
  -1.0,
  1.0,
  InterpolationSpace.Oklab,
);
brush.setPalette(Palette.concrete().dithered());

const observatory = Schematic.create("field_observatory");
BuildingTool.fill(observatory, form.toShape(), brush);
// --8<-- [end:build]


// --8<-- [start:inspect]
const size = observatory.tightDimensions();
const valueRange = field.outputRange();
const restored = Sdf.fromJsonString(form.toJson());
if (observatory.blockCount() !== 3_175) throw new Error("block count changed");
if (`${size.x},${size.y},${size.z}` !== "22,14,24") throw new Error("size changed");
if (valueRange.min !== -1 || valueRange.max !== 1) throw new Error("range changed");
if (form.evalAt(0, 0, 0) <= 0) throw new Error("shaft is not empty");
if (Math.abs(restored.evalAt(5, 2, 1) - form.evalAt(5, 2, 1)) >= 1e-6) {
  throw new Error("JSON round trip changed evaluation");
}
// --8<-- [end:inspect]

const output = process.env.SDF_FIELDS_OUT ?? "field-observatory.schem";
writeFileSync(output, Buffer.from(observatory.toSchematicB64(), "base64"));
console.log(`SDFs and fields JavaScript example: OK (${output})`);
