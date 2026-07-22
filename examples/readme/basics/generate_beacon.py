"""Generate the animated 5x5 gold beacon for docs/features/basics.md."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig

animation = BuildAnimation.create("beacon")
animation.set_step_ms(85)

# Mirror the Basics snippet exactly: 25 gold blocks, then one beacon.
for x in range(5):
    for z in range(5):
        animation.set_block(x, 0, z, "minecraft:gold_block")

animation.with_effect(AnimationEffect.spin_in(680, 1)).set_block(
    2, 1, 2, "minecraft:beacon"
)

view = RenderConfig.create(480, 360)
view.set_isometric()
view.set_yaw(28)
view.set_pitch(24)
view.set_zoom(1.22)
view.set_sphere_fit(True)
view.set_background(0, 0, 0, 0)

# A restrained drift keeps the tiny build dimensional without distracting from
# the nested-loop placement order.
camera = AnimationEffect.create(3_200)
camera.add_tween("rotateY", -5, 5, "inOutSine")
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
