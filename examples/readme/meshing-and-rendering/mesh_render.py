"""Executable Python source for docs/features/meshing-and-rendering.md."""

from base64 import b64decode
import os
from pathlib import Path

from nucleation import MeshConfig, MeshResult, RenderConfig, Renderer, ResourcePack, Schematic


# --8<-- [start:build]
scene = Schematic.create("render_lab")
scene.fill_cuboid(-5, 0, -4, 5, 0, 4, "minecraft:polished_deepslate")
scene.fill_cuboid(-4, 1, -3, 4, 1, 3, "minecraft:dark_prismarine")
for y in range(1, 5):
    for x in range(-5, 6):
        scene.set_block(x, y, -4, "minecraft:light_blue_stained_glass")
        scene.set_block(x, y, 4, "minecraft:light_blue_stained_glass")
    for z in range(-3, 4):
        scene.set_block(-5, y, z, "minecraft:light_blue_stained_glass")
        scene.set_block(5, y, z, "minecraft:light_blue_stained_glass")
for y in range(1, 4):
    scene.set_block(0, y, 0, "minecraft:sea_lantern")
scene.set_block(-3, 1, 0, "minecraft:azalea_leaves[persistent=true]")
scene.set_block(3, 1, 0, "minecraft:azalea_leaves[persistent=true]")
# --8<-- [end:build]


pack_path = Path(os.environ.get("NUCLEATION_PACK", Path(__file__).resolve().parents[3] / "render_work/pack.zip"))
pack_bytes = pack_path.read_bytes()

# --8<-- [start:mesh]
pack = ResourcePack.from_bytes(pack_bytes)
config = MeshConfig.create()
config.set_biome("lush_caves")
mesh = MeshResult.create(scene, pack, config)

glb = b64decode(mesh.glb_data_b64())
assert glb[:4] == b"glTF"
assert mesh.has_transparency()
print(mesh.vertex_count(), mesh.triangle_count())
# --8<-- [end:mesh]


# --8<-- [start:render]
view = RenderConfig.create(640, 440)
view.set_isometric()
view.set_sphere_fit(True)
view.set_background(0, 0, 0, 0)
view.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.20)
Renderer.render_to_file(scene, pack_bytes, view, "render-lab.png")
# --8<-- [end:render]


glb_out = Path(os.environ.get("MESH_RENDER_GLB_OUT", "render-lab.glb"))
schem_out = Path(os.environ.get("MESH_RENDER_SCHEM_OUT", "render-lab.schem"))
render_out = Path(os.environ.get("MESH_RENDER_PNG_OUT", "render-lab.png"))
for path in (glb_out, schem_out, render_out):
    path.parent.mkdir(parents=True, exist_ok=True)
glb_out.write_bytes(glb)
scene.save_to_file(str(schem_out))
if render_out != Path("render-lab.png"):
    Renderer.render_to_file(scene, pack_bytes, view, str(render_out))
print(f"Meshing Python example: OK ({mesh.vertex_count()} vertices, {mesh.triangle_count()} triangles)")
