import { writeFileSync, readFileSync } from "node:fs";
import { MeshConfig, MeshResult, ResourcePack, Schematic } from "nucleation";

// --8<-- [start:build]
const scene = Schematic.create("render_lab");
scene.fillCuboid(-5, 0, -4, 5, 0, 4, "minecraft:polished_deepslate");
scene.fillCuboid(-4, 1, -3, 4, 1, 3, "minecraft:dark_prismarine");
for (let y = 1; y < 5; y += 1) {
  for (let x = -5; x < 6; x += 1) {
    scene.setBlock(x, y, -4, "minecraft:light_blue_stained_glass");
    scene.setBlock(x, y, 4, "minecraft:light_blue_stained_glass");
  }
  for (let z = -3; z < 4; z += 1) {
    scene.setBlock(-5, y, z, "minecraft:light_blue_stained_glass");
    scene.setBlock(5, y, z, "minecraft:light_blue_stained_glass");
  }
}
for (let y = 1; y < 4; y += 1) scene.setBlock(0, y, 0, "minecraft:sea_lantern");
scene.setBlock(-3, 1, 0, "minecraft:azalea_leaves[persistent=true]");
scene.setBlock(3, 1, 0, "minecraft:azalea_leaves[persistent=true]");
// --8<-- [end:build]


const packBytes = [...readFileSync(process.env.NUCLEATION_PACK ?? "render_work/pack.zip")];

// --8<-- [start:mesh]
const pack = ResourcePack.fromBytes(packBytes);
const config = MeshConfig.create();
config.setBiome("lush_caves");
const mesh = MeshResult.create(scene, pack, config);

const glb = Buffer.from(mesh.glbDataB64(), "base64");
if (glb.subarray(0, 4).toString() !== "glTF") throw new Error("bad GLB header");
if (!mesh.hasTransparency()) throw new Error("transparent layer missing");
console.log(mesh.vertexCount(), mesh.triangleCount());
// --8<-- [end:mesh]


writeFileSync(process.env.MESH_RENDER_GLB_OUT ?? "render-lab.glb", glb);
writeFileSync(
  process.env.MESH_RENDER_SCHEM_OUT ?? "render-lab.schem",
  Buffer.from(scene.toSchematicB64(), "base64"),
);
console.log(`Meshing JavaScript example: OK (${mesh.vertexCount()} vertices, ${mesh.triangleCount()} triangles)`);
