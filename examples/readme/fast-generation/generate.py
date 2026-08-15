"""Generate the fast-generation guide's GIF and downloadable schematic."""

import os
from pathlib import Path

from nucleation import AnimationEffect, BuildAnimation, Diff, RenderConfig, Schematic


WIDTH = 48


def light_positions():
    positions = set()
    for p in range(0, WIDTH, 4):
        positions.update(
            {
                (p, 2, 0),
                (p, 2, WIDTH - 1),
                (0, 2, p),
                (WIDTH - 1, 2, p),
                (p, 2, WIDTH // 2),
                (WIDTH // 2, 2, p),
            }
        )
    return sorted(positions)


def towers():
    for gx in range(4, 44, 8):
        for gz in range(4, 44, 8):
            yield gx, gz, 6 + ((gx // 8 + gz // 8) % 5) * 2


def tower_block(y, height):
    if y == height + 1:
        return "minecraft:oxidized_cut_copper"
    if y % 3 == 0:
        return "minecraft:light_blue_stained_glass"
    return "minecraft:deepslate_bricks"


def build_fast():
    campus = Schematic.create("bulk_campus")
    campus.fill_cuboid(
        0, 0, 0,
        WIDTH - 1, 1, WIDTH - 1,
        "minecraft:polished_deepslate",
    )
    flat_lights = [coordinate for pos in light_positions() for coordinate in pos]
    campus.set_blocks(flat_lights, "minecraft:sea_lantern")

    indices = {
        name: campus.prepare_block(name)
        for name in (
            "minecraft:deepslate_bricks",
            "minecraft:light_blue_stained_glass",
            "minecraft:oxidized_cut_copper",
        )
    }
    for gx, gz, height in towers():
        for y in range(2, height + 2):
            index = indices[tower_block(y, height)]
            for dx in range(3):
                for dz in range(3):
                    campus.place(gx + dx, y, gz + dz, index)
    return campus


def build_animation():
    animation = BuildAnimation.create("bulk_campus")
    animation.set_step_ms(120)
    animation.set_default_effect(AnimationEffect.drop_and_pop(480, 4.0))

    # Each group is one logical bulk operation in the visual explanation.
    animation.begin_group()
    for x in range(WIDTH):
        for z in range(WIDTH):
            for y in range(2):
                animation.set_block(x, y, z, "minecraft:polished_deepslate")
    animation.end_group()

    animation.with_effect(AnimationEffect.spin_in(520, 0.75)).begin_group()
    for x, y, z in light_positions():
        animation.set_block(x, y, z, "minecraft:sea_lantern")
    animation.end_group()

    for y in range(2, 16):
        animation.begin_group()
        for gx, gz, height in towers():
            if y >= height + 2:
                continue
            for dx in range(3):
                for dz in range(3):
                    animation.set_block(gx + dx, y, gz + dz, tower_block(y, height))
        animation.end_group()

    camera = AnimationEffect.create(2_800)
    camera.add_keyframe("rotateY", 0.0, -14, "inOutSine")
    camera.add_keyframe("rotateY", 0.5, 14, "inOutSine")
    camera.add_keyframe("rotateY", 1.0, -14, "inOutSine")
    animation.animate_camera(camera, 0)
    return animation


root = Path(__file__).resolve().parents[3]
pack = Path(os.environ.get("NUCLEATION_PACK", root / "render_work/pack.zip")).read_bytes()
gif_out = Path(
    os.environ.get(
        "NUCLEATION_OUT",
        root / "docs/media/readme/fast-generation/bulk-campus.gif",
    )
)
schem_out = Path(
    os.environ.get(
        "NUCLEATION_SCHEM_OUT",
        root / "docs/downloads/readme/fast-generation/bulk-campus.schem",
    )
)
animation_schem = schem_out.with_name("bulk-campus-animation.schem")
gif_out.parent.mkdir(parents=True, exist_ok=True)
schem_out.parent.mkdir(parents=True, exist_ok=True)

campus = build_fast()
animation = build_animation()
campus.save_to_file(str(schem_out))
animation.save_to_file(str(animation_schem))

animated = Schematic.load_from_file(str(animation_schem))
assert Diff.compute(campus, animated, "exact").distance() == 0
animation_schem.unlink()

config = RenderConfig.create(460, 380)
config.set_isometric()
config.set_sphere_fit(True)
config.set_background(0, 0, 0, 0)
config.set_fitted_grid(4, 1, -0.502, False, 0.42, 0.52, 0.60, 0.24)

frames = animation.render_gif(pack, config, str(gif_out), 15, 700)
print(f"saved {schem_out}")
print(f"rendered {frames} frames to {gif_out}")
