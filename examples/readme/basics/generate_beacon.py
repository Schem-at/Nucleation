"""Generate the animated 3x3 gold beacon on a 5x5 Cartesian grid."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig

animation = BuildAnimation.create("beacon")
animation.set_step_ms(140)

# Mirror the Basics snippet exactly: nine gold blocks, then one beacon.
for x in range(-1, 2):
    for z in range(-1, 2):
        animation.set_block(x, 0, z, "minecraft:gold_block")

animation.with_effect(AnimationEffect.spin_in(680, 1)).set_block(
    0, 1, 0, "minecraft:beacon"
)

view = RenderConfig.create(480, 360)
view.set_isometric()
view.set_yaw(28)
view.set_pitch(24)
view.set_zoom(0.80)
view.set_sphere_fit(True)
view.set_background(0, 0, 0, 0)
view.set_grid(2, 1, -0.502, True, 0.44, 0.54, 0.66, 0.32)

# A restrained drift keeps the tiny build dimensional without distracting from
# the nested-loop placement order.
camera = AnimationEffect.create(2_400)
camera.add_tween("rotateY", -4, 4, "inOutSine")
animation.animate_camera(camera, 0)

root = Path(__file__).resolve().parents[3]
pack = Path(
    os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")
).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/basics/beacon.gif",
    )
)
schem_out = Path(
    os.environ.get(
        "NUCLEATION_SCHEM_OUT",
        root / "docs/downloads/readme/basics/beacon.schem",
    )
)
gif_out.parent.mkdir(parents=True, exist_ok=True)
schem_out.parent.mkdir(parents=True, exist_ok=True)

animation.save_to_file(str(schem_out))
frames = animation.render_gif(pack, view, str(gif_out), 18, 900)
print(f"saved {schem_out}")
print(f"rendered {frames} frames to {gif_out}")
