"""README workshop built and animated entirely through the public Python API."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig

animation = BuildAnimation.create("workshop")

# One animation target for the whole nested-loop floor.
animation.begin_group()
for x in range(-3, 3):
    for z in range(-2, 3):
        animation.set_block(x, 0, z, "minecraft:oak_planks")
animation.end_group()

# Normal calls use drop-and-pop; one call overrides it.
spin = AnimationEffect.spin_in(600, 1)
animation.with_effect(spin).set_block(
    -2, 1, -1, "minecraft:furnace[facing=south]"
)
animation.set_block(-1, 1, -1, "minecraft:crafting_table")
animation.set_block(1, 1, -1, "minecraft:chest[facing=south]")
animation.add_armor_stand(0.5, 1, -0.5, 0, "diamond")

# Camera animation uses the same effect/track model as blocks.
camera = AnimationEffect.create(3_000)
camera.add_tween("rotateY", -8, 8, "inOutSine")
animation.animate_camera(camera, 0)

config = RenderConfig.create(420, 420)
config.set_isometric()
config.set_zoom(1.12)
config.set_sphere_fit(True)
config.set_background(0, 0, 0, 0)
# Floor blocks are centred at y=0 and end at y=-0.5. Put the grid just below
# their bottom faces rather than through their vertical centres.
config.set_fitted_grid(1, 1, -0.502, False, 0.42, 0.52, 0.60, 0.26)

root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/animation/workshop.gif",
    )
)
schem_out = Path(
    os.environ.get(
        "NUCLEATION_SCHEM_OUT",
        root / "docs/downloads/readme/animation/workshop.schem",
    )
)
gif_out.parent.mkdir(parents=True, exist_ok=True)
schem_out.parent.mkdir(parents=True, exist_ok=True)

frames = animation.render_gif(pack, config, str(gif_out), 18, 1_000)
animation.save_to_file(str(schem_out))
print(f"saved {schem_out}")
print(f"rendered {frames} frames to {gif_out}")
