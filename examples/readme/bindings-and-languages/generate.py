"""Generate the bindings guide media and download."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig, Renderer, Schematic


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(os.environ.get(
    "NUCLEATION_OUT",
    root / "docs/media/readme/bindings-and-languages/binding-stack.gif",
))
still_out = Path(os.environ.get(
    "NUCLEATION_STILL_OUT",
    root / "docs/media/readme/bindings-and-languages/binding-stack.png",
))
schem_out = Path(os.environ.get(
    "NUCLEATION_SCHEM_OUT",
    root / "docs/downloads/readme/bindings-and-languages/binding-stack.schem",
))
for path in (gif_out, still_out, schem_out):
    path.parent.mkdir(parents=True, exist_ok=True)

stack = Schematic.create("binding_stack")
stack.fill_cuboid(-3, 0, -3, 3, 0, 3, "minecraft:polished_deepslate")
stack.fill_cuboid(-2, 1, -2, 2, 1, 2, "minecraft:light_blue_concrete")
stack.fill_cuboid(-1, 2, -1, 1, 2, 1, "minecraft:yellow_concrete")
stack.set_block(0, 3, 0, "minecraft:emerald_block")
stack.save_to_file(str(schem_out))

still = RenderConfig.create(560, 420)
still.set_isometric()
still.set_sphere_fit(True)
still.set_background(0, 0, 0, 0)
still.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
Renderer.render_to_file(stack, pack, still, str(still_out))

animation = BuildAnimation.create("binding_stack")
animation.set_default_effect(AnimationEffect.drop_and_pop(480, 2.8))
animation.set_stagger_total_ms(1_900)
layers = [
    ((-3, 0, -3, 3, 0, 3), "minecraft:polished_deepslate"),
    ((-2, 1, -2, 2, 1, 2), "minecraft:light_blue_concrete"),
    ((-1, 2, -1, 1, 2, 1), "minecraft:yellow_concrete"),
]
for (x1, y1, z1, x2, y2, z2), block in layers:
    animation.begin_group()
    for y in range(y1, y2 + 1):
        for x in range(x1, x2 + 1):
            for z in range(z1, z2 + 1):
                animation.set_block(x, y, z, block)
    animation.end_group()
animation.set_block(0, 3, 0, "minecraft:emerald_block")

camera = AnimationEffect.create(3_000)
camera.add_tween("rotateY", -10, 10, "inOutSine")
animation.animate_camera(camera, 0)

motion = RenderConfig.create(460, 380)
motion.set_isometric()
motion.set_sphere_fit(True)
motion.set_background(0, 0, 0, 0)
motion.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.22)
frames = animation.render_gif(pack, motion, str(gif_out), 15, 700)

print(f"saved {schem_out}")
print(f"rendered still to {still_out}")
print(f"rendered {frames} frames to {gif_out}")
