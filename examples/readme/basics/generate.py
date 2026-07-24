"""A compact crafting nook for docs/features/basics.md."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, RenderConfig

animation = BuildAnimation.create("crafting_nook")
animation.set_step_ms(520)

# A warm five-by-five floor arrives as one readable construction step.
animation.begin_group()
for x in range(5):
    for z in range(5):
        animation.set_block(x, 0, z, "minecraft:spruce_planks")
animation.end_group()

# Two three-high walls make a real corner, each with one centred window.
animation.begin_group()
for y in (1, 2, 3):
    for x in range(5):
        if x == 2 and y == 2:
            block = "minecraft:light_blue_stained_glass"
        elif x in (0, 4):
            block = "minecraft:stripped_spruce_log[axis=y]"
        else:
            block = "minecraft:oak_planks"
        animation.set_block(x, y, 0, block)

    for z in range(1, 5):
        if z == 2 and y == 2:
            block = "minecraft:light_blue_stained_glass"
        elif z == 4:
            block = "minecraft:stripped_spruce_log[axis=y]"
        else:
            block = "minecraft:oak_planks"
        animation.set_block(0, y, z, block)
animation.end_group()

# The focal object gets one playful entrance; everything else stays restrained.
animation.with_effect(AnimationEffect.spin_in(620, 1)).set_block(
    1, 1, 1, "minecraft:crafting_table"
)
animation.set_block(3, 1, 1, "minecraft:chest[facing=south]")

# Wall torches occupy the air beside the end posts, not the wall blocks themselves.
animation.begin_group()
animation.set_block(4, 2, 1, "minecraft:wall_torch[facing=south]")
animation.set_block(1, 2, 4, "minecraft:wall_torch[facing=east]")
animation.end_group()

view = RenderConfig.create(480, 420)
view.set_isometric()
view.set_zoom(0.98)
view.set_sphere_fit(True)
view.set_background(0, 0, 0, 0)
view.set_fitted_grid(1, 1, -0.502, False, 0.44, 0.54, 0.66, 0.24)

camera = AnimationEffect.create(3_000)
camera.add_tween("rotateY", -5, 6, "inOutSine")
animation.animate_camera(camera, 0)

root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/basics/animation.gif",
    )
)
schem_out = Path(
    os.environ.get(
        "NUCLEATION_SCHEM_OUT",
        root / "docs/downloads/readme/basics/crafting-nook.schem",
    )
)
gif_out.parent.mkdir(parents=True, exist_ok=True)
schem_out.parent.mkdir(parents=True, exist_ok=True)

animation.save_to_file(str(schem_out))
frames = animation.render_gif(pack, view, str(gif_out), 18, 1_000)
print(f"saved {schem_out}")
print(f"rendered {frames} frames to {gif_out}")
