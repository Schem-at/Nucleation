"""Render the signed-coordinate illustration used by the Basics page."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig


animation = BuildAnimation.create("signed_coordinate_axes")
animation.set_step_ms(230)
animation.with_effect(AnimationEffect.spin_in(620, 1)).set_block(
    0, 0, 0, "minecraft:gold_block"
)

axes = (
    ((1, 0, 0), "minecraft:red_concrete"),
    ((-1, 0, 0), "minecraft:blue_concrete"),
    ((0, 0, 1), "minecraft:orange_concrete"),
    ((0, 0, -1), "minecraft:purple_concrete"),
    ((0, 1, 0), "minecraft:lime_concrete"),
)
for (dx, dy, dz), block in axes:
    animation.begin_group()
    for distance in range(1, 5):
        animation.set_block(dx * distance, dy * distance, dz * distance, block)
    animation.end_group()

view = RenderConfig.create(480, 390)
view.set_isometric()
view.set_yaw(34)
view.set_pitch(26)
view.set_zoom(0.92)
view.set_sphere_fit(True)
view.set_background(0, 0, 0, 0)
view.set_grid(5, 1, -0.502, True, 0.44, 0.54, 0.66, 0.25)

camera = AnimationEffect.create(2_600)
camera.add_tween("rotateY", -5, 5, "inOutSine")
animation.animate_camera(camera, 0)

root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/basics/coordinates.gif",
    )
)
gif_out.parent.mkdir(parents=True, exist_ok=True)

frames = animation.render_gif(pack, view, str(gif_out), 18, 900)
print(f"rendered {frames} frames to {gif_out}")
