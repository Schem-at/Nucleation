"""Generate the meshing and rendering guide media and downloads."""

from base64 import b64decode
import os
from pathlib import Path

from nucleation import (
    AnimationEffect,
    BuildAnimation,
    MeshConfig,
    MeshResult,
    RenderConfig,
    Renderer,
    ResourcePack,
    Schematic,
)


def make_scene():
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
    return scene


root = Path(__file__).resolve().parents[3]
pack_bytes = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/meshing-and-rendering/render-lab.png",
))
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/meshing-and-rendering/render-lab-turntable.gif",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/meshing-and-rendering/render-lab.schem",
))
glb_out = Path(os.environ.get(
    "NUCLEATION_GLB_OUT",
    root / "docs/downloads/readme/meshing-and-rendering/render-lab.glb",
))
for path in (still_out, gif_out, schem_out, glb_out):
    path.parent.mkdir(parents=True, exist_ok=True)

scene = make_scene()
scene.save_to_file(str(schem_out))
pack = ResourcePack.from_bytes(pack_bytes)
config = MeshConfig.create()
config.set_biome("lush_caves")
mesh = MeshResult.create(scene, pack, config)
glb_out.write_bytes(b64decode(mesh.glb_data_b64()))

still = RenderConfig.create(720, 480)
still.set_isometric()
still.set_sphere_fit(True)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.20)
Renderer.render_to_file(scene, pack_bytes, still, str(still_out))

animation = BuildAnimation.from_schematic(scene)
animation.animate_all(AnimationEffect.instant())
animation.animate_camera(AnimationEffect.turntable(3_200), 0)
animation.set_loop_period_ms(3_200)

motion = RenderConfig.create(460, 380)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.20)
frames = animation.render_gif(pack_bytes, motion, str(gif_out), 15, 0)

print(f"saved {schem_out} and {glb_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
